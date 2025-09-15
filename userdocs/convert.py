import os
import re
from lxml import etree

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

def process_xml(file: str) -> str:
    print("Loading", file)
    with open(file, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()
    # Удаляем DOCTYPE и любые сущности, чтобы избежать ошибок парсинга
    content = re.sub(r'<!DOCTYPE\s+[^>]*?(?:\s*\[.*?\]\s*)?>|<!DOCTYPE\s+[^>]*>', '', content, flags=re.IGNORECASE | re.DOTALL)
    # Также удаляем любые объявления сущностей
    content = re.sub(r'<!ENTITY\s+[^>]*>', '', content, flags=re.IGNORECASE | re.DOTALL)
    parser = etree.XMLParser(dtd_validation=False, load_dtd=False, no_network=True, recover=True, remove_blank_text=True)
    try:
        root = etree.fromstring(content.encode("utf-8"), parser=parser)
    except etree.XMLSyntaxError as e:
        print(f"XML parse error: {e}")
        return ""
    md_lines = []
    build_markdown(root, md_lines, level=1)
    return "\n".join(md_lines).strip()

def build_markdown(elem, lines, level=1):
    """Convert XML element to Markdown with tag semantics."""
    tag = elem.tag.lower()
    text = (elem.text or "").strip()
    tail = (elem.tail or "").strip()

    if tag == 'section':
        # Обрабатываем секции как подразделы с заголовками
        title_elem = elem.find('title')
        if title_elem is not None:
            lines.append(f"{'#' * (level + 1)} {title_elem.text.strip() if title_elem.text else ''}")
            lines.append("")
    elif tag == 'title':
        # Стандартный заголовок
        lines.append(f"{'#' * level} {text}")
        lines.append("")
    elif tag == 'titleabbrev':
        # Короткий заголовок как основной
        lines.append(f"# {text}")
        lines.append("")
    elif tag == 'para':
        # Параграф с поддержкой вложенных элементов
        para_text = text
        for child in elem:
            child_md = get_inline_markdown(child)
            para_text += child_md
            if child.tail:
                para_text += child.tail.strip()
        lines.append(para_text)
        lines.append("")
    elif tag == 'funcsynopsis':
        # Синопсис функции
        role = elem.get('role', '').capitalize()
        lines.append(f"### Синтаксис ({role}):")
        lines.append("")
    elif tag == 'funcsynopsisinfo':
        # Информация о синопсисе
        package = elem.find('package')
        if package is not None:
            lines.append(f"**Package:** {package.text.strip() if package.text else ''}")
            lines.append("")
    elif tag == 'funcprototype':
        # Прототип функции с улучшенным форматированием
        func_def_element = elem.find('funcdef')
        if func_def_element is not None:
            function_name = func_def_element.find('function').text.strip() if func_def_element.find('function') is not None else ""
            return_type = (func_def_element.text or "").strip()
            param_list = []
            for param in elem.findall('paramdef'):
                param_type = (param.text or "").strip()
                param_name_elem = param.find('parameter')
                param_name = param_name_elem.text.strip() if param_name_elem is not None else ""
                param_list.append(f"{param_type} {param_name}".strip())
            params_str = ", ".join(param_list)
            lines.append(f"**{return_type} `{function_name}`({params_str})**")
            lines.append("")
    elif tag == 'example':
        # Пример с заголовком
        title_element = elem.find('title')
        title_text = title_element.text.strip() if title_element is not None and title_element.text else "Пример"
        lines.append(f"### {title_text}")
        lines.append("")
    elif tag == 'programlisting':
        # Код с языком
        lang = elem.get('role', '').lower() or "text"
        code = etree.tostring(elem, method='text', encoding='unicode').strip()
        lines.append(f"```{lang}")
        lines.append(code)
        lines.append("```")
        lines.append("")
    elif tag == 'article':
        # Корневой элемент, просто проходим детей
        pass
    elif tag == 'package':
        # Пакет
        lines.append(f"**Package:** {text}")
        lines.append("")
    elif tag == 'itemizedlist':
        # Маркированный список
        lines.append("")  # Разделитель перед списком
    elif tag == 'listitem':
        # Элемент списка
        item_text = text
        for child in elem:
            if child.tag.lower() == 'para':
                item_text += get_inline_markdown(child, is_para=True)
            else:
                item_text += get_inline_markdown(child)
            if child.tail:
                item_text += child.tail.strip()
        lines.append(f"- {item_text}")
        lines.append("")
    elif tag == 'variablelist':
        # Список переменных (как dl в HTML)
        lines.append("")
    elif tag == 'varlistentry':
        # Элемент variablelist
        term = elem.find('term')
        listitem = elem.find('listitem')
        if term is not None:
            term_text = term.text.strip() if term.text else ""
            lines.append(f"**{term_text}**")
        if listitem is not None:
            item_text = ""
            for child in listitem:
                item_text += get_inline_markdown(child, is_para=True if child.tag.lower() == 'para' else False)
                if child.tail:
                    item_text += child.tail.strip()
            lines.append(item_text)
            lines.append("")
    elif tag == 'emphasis':
        # Emphasis с ролями
        role = elem.get('role', '').lower()
        inner_text = etree.tostring(elem, method='text', encoding='unicode').strip()
        if role == 'bold':
            lines.append(f"**{inner_text}**")
        elif role == 'italic':
            lines.append(f"*{inner_text}*")
        else:
            lines.append(inner_text)
    elif tag == 'literal' or tag == 'code':
        # Inline code
        inner_text = etree.tostring(elem, method='text', encoding='unicode').strip()
        lines.append(f"`{inner_text}`")
    elif tag == 'link':
        # Ссылки
        href = elem.get('xlink:href', '')  # Предполагаем namespace xlink
        inner_text = etree.tostring(elem, method='text', encoding='unicode').strip()
        lines.append(f"[{inner_text}]({href})")
    else:
        # Для неизвестных тегов - просто текст
        if text:
            lines.append(text)
        print(f"Unknown tag: {tag}")

    # Рекурсивно обрабатываем детей, если не обработаны inline
    for child in elem:
        build_markdown(child, lines, level + 1)

    # Добавляем tail, если есть
    if tail:
        lines.append(tail)
        lines.append("")

def get_inline_markdown(elem, is_para=False):
    """Получаем inline Markdown для вложенных элементов."""
    tag = elem.tag.lower()
    text = (elem.text or "").strip()
    md = ""
    if tag == 'emphasis':
        role = elem.get('role', '').lower()
        inner = text + ''.join(get_inline_markdown(c) for c in elem) + (elem.tail or "").strip()
        if role == 'bold':
            md = f"**{inner}**"
        elif role == 'italic':
            md = f"*{inner}*"
        else:
            md = inner
    elif tag == 'literal' or tag == 'code':
        md = f"`{text}`"
    elif tag == 'link':
        href = elem.get('xlink:href', '')
        inner = text + ''.join(get_inline_markdown(c) for c in elem)
        md = f"[{inner}]({href})"
    elif tag == 'parameter' or tag == 'function':
        md = f"`{text}`"
    elif tag == 'para' and is_para:
        md = text + ''.join(get_inline_markdown(c) for c in elem)
    else:
        md = text + ''.join(get_inline_markdown(c) for c in elem)
    return md

# Example usage:
target_directory = 'userdocs_old_xml'
process_files_in_tree(target_directory)
