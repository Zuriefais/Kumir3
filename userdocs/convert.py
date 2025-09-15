import os
import re
from lxml import etree
from collections import defaultdict

def process_files_in_tree(root_dir, out_dir="md_output"):
    """
    Traverse directory tree, convert XML to Markdown, save as .md files.
    """
    os.makedirs(out_dir, exist_ok=True)
    for root, dirs, files in os.walk(root_dir):
        for filename in files:
            if filename.lower().endswith(".xml"):
                file_path = os.path.join(root, filename)
                try:
                    md_text = process_xml(file_path)
                    out_file = os.path.join(out_dir, os.path.splitext(filename)[0] + ".md")
                    with open(out_file, "w", encoding="utf-8") as f:
                        f.write(md_text)
                    print(f"✅ {file_path} -> {out_file}")
                except Exception as e:
                    print(f"⚠️ Error with {file_path}: {e}")

def process_single_file(input_file, output_file=None):
    """
    Convert a single XML file to Markdown, useful for debugging.
    If output_file is not specified, saves to the same directory with .md extension.
    """
    if not input_file.lower().endswith(".xml"):
        print(f"⚠️ Error: {input_file} is not an XML file")
        return
    try:
        md_text = process_xml(input_file)
        if output_file is None:
            output_file = os.path.splitext(input_file)[0] + ".md"
        with open(output_file, "w", encoding="utf-8") as f:
            f.write(md_text)
        print(f"✅ {input_file} -> {output_file}")
    except Exception as e:
        print(f"⚠️ Error with {input_file}: {e}")

def process_xml(file: str) -> str:
    print("Loading", file)
    with open(file, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()
    # Удаляем DOCTYPE и сущности, чтобы избежать ошибок парсинга
    content = re.sub(r'<!DOCTYPE\s+[^>]*?(?:\s*\[.*?\]\s*)?>|<!DOCTYPE\s+[^>]*>', '', content, flags=re.IGNORECASE | re.DOTALL)
    content = re.sub(r'<!ENTITY\s+[^>]*>', '', content, flags=re.IGNORECASE | re.DOTALL)
    parser = etree.XMLParser(dtd_validation=False, load_dtd=False, no_network=True, recover=True, remove_blank_text=True)
    try:
        root = etree.fromstring(content.encode("utf-8"), parser=parser)
    except etree.XMLSyntaxError as e:
        print(f"XML parse error: {e}")
        return ""
    md_lines = []
    unknown_tags = defaultdict(lambda: {'count': 0, 'attrs': set(), 'parents': set()})
    build_markdown(root, md_lines, level=1, unknown_tags=unknown_tags)
    # Добавляем таблицу неизвестных тегов в конец Markdown, если они есть
    if unknown_tags:
        md_lines.append("\n## Неизвестные теги\n")
        md_lines.append("| Тег | Количество | Атрибуты | Родительские теги |")
        md_lines.append("|-----|------------|----------|-------------------|")
        for tag, info in sorted(unknown_tags.items()):
            attrs_str = ", ".join(sorted([f"{k}='{v}'" for k, v in info['attrs']])) if info['attrs'] else ""
            parents_str = ", ".join(sorted(info['parents'])) if info['parents'] else "None"
            md_lines.append(f"| {tag} | {info['count']} | {attrs_str} | {parents_str} |")
    return "\n".join(md_lines).strip()

def build_markdown(elem, lines, level=1, unknown_tags=None, parent_tag=None):
    """Convert XML element to Markdown with tag semantics, track unknown tags."""
    if unknown_tags is None:
        unknown_tags = defaultdict(lambda: {'count': 0, 'attrs': set(), 'parents': set()})
    tag = elem.tag.lower()
    text = (elem.text or "").strip()
    tail = (elem.tail or "").strip()

    # Список тегов, которые обрабатываются в составе других тегов и не должны быть неизвестными
    handled_tags = {'funcdef', 'function', 'paramdef', 'parameter', 'type', 'title'}

    if tag == 'article':
        for child in elem:
            build_markdown(child, lines, level, unknown_tags, tag)
    elif tag == 'section':
        title_elem = elem.find('title')
        if title_elem is not None:
            title_text = get_inline_markdown(title_elem)
            lines.append(f"{'#' * (level + 1)} {title_text}")
            lines.append("")
        for child in elem:
            if child.tag.lower() != 'title':
                build_markdown(child, lines, level + 1, unknown_tags, tag)
    elif tag == 'title' or tag == 'titleabbrev':
        # Заголовок уже обработан в section
        if parent_tag not in ['section', 'example']:
            lines.append(f"{'#' * level} {text}")
            lines.append("")
    elif tag == 'para':
        para_text = get_inline_markdown(elem)
        if para_text:
            lines.append(para_text)
            lines.append("")
    elif tag == 'funcsynopsis':
        role = elem.get('role', '').capitalize()
        lines.append(f"### Синтаксис ({role}):")
        lines.append("")
        for child in elem:
            build_markdown(child, lines, level + 1, unknown_tags, tag)
    elif tag == 'funcsynopsisinfo':
        package = elem.find('package')
        if package is not None:
            package_text = get_inline_markdown(package)
            if package_text:
                lines.append(f"**Пакет:** {package_text}")
                lines.append("")
        for child in elem:
            build_markdown(child, lines, level + 1, unknown_tags, tag)
    elif tag == 'funcprototype':
        func_def_element = elem.find('funcdef')
        if func_def_element is not None:
            function_name = (func_def_element.find('function').text or "").strip()
            return_type_elem = func_def_element.find('type')
            return_type = (return_type_elem.text or "").strip() if return_type_elem is not None else (func_def_element.text or "").strip()
            param_list = []
            for param in elem.findall('paramdef'):
                param_type = (param.find('type').text or "").strip() if param.find('type') is not None else ""
                param_name = (param.find('parameter').text or "").strip() if param.find('parameter') is not None else ""
                param_list.append(f"{param_type} `{param_name}`".strip())
            params_str = ", ".join(param_list)
            lines.append(f"**{return_type} `{function_name}` ({params_str})**")
            lines.append("")
    elif tag == 'example':
        title_element = elem.find('title')
        title_text = get_inline_markdown(title_element) if title_element is not None else "Пример"
        lines.append(f"### {title_text}")
        lines.append("")
        for child in elem:
            if child.tag.lower() != 'title':
                build_markdown(child, lines, level + 1, unknown_tags, tag)
    elif tag == 'programlisting':
        lang = elem.get('role', '').lower() or "text"
        code = etree.tostring(elem, method='text', encoding='unicode').strip()
        lines.append(f"```{lang}")
        lines.append(code)
        lines.append("```")
        lines.append("")
    elif tag == 'itemizedlist' or tag == 'orderedlist':
        lines.append("")
        for child in elem:
            build_markdown(child, lines, level, unknown_tags, tag)
    elif tag == 'listitem':
        parent_is_ordered = parent_tag == 'orderedlist'
        prefix = "1." if parent_is_ordered else "*"
        item_text = get_inline_markdown(elem)
        if item_text:
            lines.append(f"{prefix} {item_text}")
            lines.append("")
    elif tag == 'variablelist':
        lines.append("")
        for child in elem:
            build_markdown(child, lines, level, unknown_tags, tag)
    elif tag == 'varlistentry':
        term = elem.find('term')
        listitem = elem.find('listitem')
        if term is not None:
            term_text = get_inline_markdown(term)
            lines.append(f"**{term_text}**")
        if listitem is not None:
            item_text = get_inline_markdown(listitem)
            if item_text:
                lines.append(item_text)
                lines.append("")
    elif tag in handled_tags:
        pass
    else:
        # Регистрируем неизвестный тег
        unknown_tags[tag]['count'] += 1
        if elem.attrib:
            unknown_tags[tag]['attrs'].update((k, v) for k, v in elem.attrib.items())
        unknown_tags[tag]['parents'].add(parent_tag or 'None')
        print(f"Unknown tag: {tag}")

        # Добавляем контент неизвестного тега
        content = get_inline_markdown(elem)
        if content:
            lines.append(content)
            lines.append("")
        for child in elem:
            if child.tag.lower() not in handled_tags:
                build_markdown(child, lines, level + 1, unknown_tags, tag)

    if tail and parent_tag not in ['para', 'emphasis', 'literal', 'code', 'link', 'listitem', 'varlistentry', 'title', 'funcprototype']:
        lines.append(tail)
        lines.append("")

def get_inline_markdown(elem):
    """Recursively process inline elements and their text."""
    md_parts = []

    if elem.text and elem.text.strip():
        md_parts.append(elem.text.strip())

    for child in elem:
        tag = child.tag.lower()
        sub_content = get_inline_markdown(child)

        if tag == 'emphasis':
            role = child.get('role', '').lower()
            if role == 'bold':
                md_parts.append(f"**{sub_content}**")
            elif role == 'italic':
                md_parts.append(f"*{sub_content}*")
            else:
                md_parts.append(sub_content)
        elif tag in ['literal', 'code', 'parameter', 'function']:
            md_parts.append(f"`{sub_content}`")
        elif tag == 'link':
            href = child.get('{[http://www.w3.org/1999/xlink](http://www.w3.org/1999/xlink)}href', '')
            md_parts.append(f"[{sub_content}]({href})")
        elif tag == 'type':
            md_parts.append(sub_content)
        else:
            md_parts.append(sub_content)

        if child.tail and child.tail.strip():
            md_parts.append(child.tail.strip())

    return "".join(md_parts)

# Example usage for single file:
input_file = 'userdocs_old_xml/ActorRobot.xml'
output_file = 'robot.md'
process_single_file(input_file, output_file)

process_files_in_tree("userdocs_old_xml/")
