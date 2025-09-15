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
    content = re.sub(r'<!DOCTYPE\s+[^>]*?(?:\s*\[.*?\]\s*)?>|<!DOCTYPE\s+[^>]*>', '', content, flags=re.IGNORECASE | re.DOTALL)
    parser = etree.XMLParser(dtd_validation=False, load_dtd=False, no_network=True, recover=True)
    root = etree.fromstring(content.encode("utf-8"), parser=parser)
    md_lines = []
    build_markdown(root, md_lines, level=1)
    return "\n".join(md_lines)

def build_markdown(elem, lines, level=1):
    """Convert XML element to Markdown with tag semantics."""
    tag = elem.tag.lower()
    if tag == 'section':
        pass
    elif tag == 'title':
        lines.append(f"## {elem.text.strip()}")
        lines.append("")
    elif tag == 'titleabbrev':
        lines.append(f"# {elem.text.strip()}")
        lines.append("")
    elif tag == 'para':
        para_text = elem.text.strip() if elem.text else ""
        for child in elem:
            if child.tag.lower() == 'emphasis' and child.get('role') == 'italic':
                para_text += f" *{child.text.strip()}*"
                if child.tail:
                    para_text += child.tail.strip()
        lines.append(para_text)
        lines.append("")
    elif tag == 'funcsynopsis':
        role = elem.get('role', '').capitalize()
        lines.append(f"### Синтаксис ({role}):")
    elif tag == 'funcsynopsisinfo':
        package = elem.find('package')
        if package is not None:
            lines.append(f"**Package:** {package.text.strip()}")
            lines.append("")
    elif tag == 'funcprototype':
        func_def_element = elem.find('funcdef')
        if func_def_element is not None:
            function_name = func_def_element.find('function').text
            full_text = etree.tostring(func_def_element, method='text', encoding='unicode').strip()
            param_text = ""
            for param in elem.findall('paramdef'):
                param_name = param.find('parameter').text if param.find('parameter') is not None else ""
                param_type = param.text.strip() if param.text else ""
                param_text += f"{param_type} {param_name}".strip() + ", "
            param_text = param_text.rstrip(", ")
            lines.append(f"`{full_text} ({param_text})`**`{function_name}`**")
            lines.append("")
    elif tag == 'example':
        title_element = elem.find('title')
        lines.append(f"### {title_element.text.strip()}" if title_element is not None else "### Пример")
    elif tag == 'programlisting':
        lang = elem.get('role', '')
        code = elem.text.strip() if elem.text else ""
        lines.append(f"```{lang}")
        lines.append(code)
        lines.append("```")
        lines.append("")
    elif tag == 'article':
        pass
    elif tag == 'package':
        lines.append(f"**Package:** {elem.text.strip()}")
        lines.append("")
    elif tag == 'itemizedlist':
        pass
    elif tag == 'listitem':
        param = elem.find('parameter')
        param_text = f"{param.text.strip()}" if param is not None else ""
        text = elem.text.strip() if elem.text else ""
        tail = elem.find('parameter').tail.strip() if param is not None and elem.find('parameter').tail else ""
        lines.append(f"- **{param_text}**{tail}")
        lines.append("")
    else:
        print("unknown tag", tag)
    for child in elem:
        if tag in ['example', 'funcprototype', 'funcsynopsisinfo', 'para', 'itemizedlist', 'listitem'] and child.tag.lower() in ['title', 'funcdef', 'package', 'emphasis', 'parameter']:
            continue
        build_markdown(child, lines, level + 1)
    if elem.tail and elem.tail.strip():
        lines.append(elem.tail.strip())

# Example usage:
target_directory = 'userdocs_old_xml'
process_files_in_tree(target_directory)
