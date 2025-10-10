#!/usr/bin/env python3
"""
Automatically add @spec tags to source code files based on TRACEABILITY_MATRIX.md
This script parses the traceability matrix and adds specification tags to all code files.
"""

import re
import os
from pathlib import Path
from typing import Dict, List, Tuple
from collections import defaultdict

# Project root
PROJECT_ROOT = Path(__file__).parent.parent
TRACEABILITY_FILE = PROJECT_ROOT / "specs" / "TRACEABILITY_MATRIX.md"

# Mapping of requirements to their details
requirements_to_code: Dict[str, List[Tuple[str, str, str, str]]] = defaultdict(list)

def parse_traceability_matrix():
    """Parse TRACEABILITY_MATRIX.md to extract code locations and test mappings"""
    print("📖 Parsing TRACEABILITY_MATRIX.md...")

    with open(TRACEABILITY_FILE, 'r', encoding='utf-8') as f:
        content = f.read()

    # Find the "Requirements to Code Mapping" section
    rust_section = re.search(r'### Rust Core Engine\s+\|(.*?)\n\n###', content, re.DOTALL)
    python_section = re.search(r'### Python AI Service\s+\|(.*?)\n\n###', content, re.DOTALL)
    frontend_section = re.search(r'### Next\.js Dashboard\s+\|(.*?)\n\n---', content, re.DOTALL)

    sections = {
        'Rust': rust_section.group(1) if rust_section else "",
        'Python': python_section.group(1) if python_section else "",
        'Frontend': frontend_section.group(1) if frontend_section else "",
    }

    for lang, section_content in sections.items():
        # Parse table rows
        rows = re.findall(r'\|\s*([^\|]+?)\s*\|\s*`([^`]+?)`\s*\|\s*([^\|]+?)\s*\|', section_content)
        for req_id, code_location, status in rows:
            req_id = req_id.strip()
            code_location = code_location.strip()
            status = status.strip()

            # Extract file path and line range
            if ':' in code_location:
                file_path, line_range = code_location.rsplit(':', 1)
                file_path = PROJECT_ROOT / file_path
                requirements_to_code[str(file_path)].append((req_id, line_range, lang, status))

    print(f"✅ Found {len(requirements_to_code)} files with requirements mappings")
    return requirements_to_code


def get_comment_syntax(file_path: str) -> str:
    """Get comment syntax based on file extension"""
    if file_path.endswith('.rs'):
        return '//'
    elif file_path.endswith('.py'):
        return '#'
    elif file_path.endswith(('.ts', '.tsx', '.js', '.jsx')):
        return '//'
    return '#'


def get_spec_doc_ref(req_id: str) -> str:
    """Get the specification document reference for a requirement"""
    mapping = {
        'FR-AUTH': 'specs/02-design/2.5-components/COMP-RUST-AUTH.md',
        'FR-TRADING': 'specs/02-design/2.5-components/COMP-RUST-TRADING.md',
        'FR-RISK': 'specs/02-design/2.5-components/COMP-RUST-TRADING.md#risk-management',
        'FR-PORTFOLIO': 'specs/02-design/2.5-components/COMP-RUST-TRADING.md#portfolio',
        'FR-PAPER': 'specs/02-design/2.5-components/COMP-RUST-TRADING.md#paper-trading',
        'FR-STRATEGY': 'specs/02-design/2.5-components/COMP-RUST-TRADING.md#strategies',
        'FR-WEBSOCKET': 'specs/02-design/2.3-api/API-WEBSOCKET.md',
        'FR-MARKET': 'specs/02-design/2.3-api/API-RUST-CORE.md#market-data',
        'FR-AI': 'specs/02-design/2.5-components/COMP-PYTHON-ML.md',
        'FR-DASHBOARD': 'specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md',
    }

    for prefix, doc in mapping.items():
        if req_id.startswith(prefix):
            return doc
    return 'specs/README.md'


def get_test_cases(req_id: str) -> str:
    """Get test case IDs for a requirement from traceability matrix"""
    # Read traceability matrix to find test cases
    with open(TRACEABILITY_FILE, 'r', encoding='utf-8') as f:
        content = f.read()

    # Find the row with this requirement ID
    pattern = rf'\|\s*{re.escape(req_id)}\s*\|[^\|]*?\|[^\|]*?\|\s*([^\|]+?)\s*\|[^\|]*?\|'
    match = re.search(pattern, content)
    if match:
        test_cases = match.group(1).strip()
        # Clean up test case references
        test_cases = re.sub(r'\s+', ' ', test_cases)
        return test_cases
    return "N/A"


def get_requirement_description(req_id: str) -> str:
    """Get requirement description from traceability matrix"""
    with open(TRACEABILITY_FILE, 'r', encoding='utf-8') as f:
        content = f.read()

    # Find the row with this requirement ID
    pattern = rf'\|\s*{re.escape(req_id)}\s*\|\s*([^\|]+?)\s*\|'
    match = re.search(pattern, content)
    if match:
        description = match.group(1).strip()
        return description
    return "Unknown requirement"


def create_spec_tag(req_id: str, comment_char: str) -> str:
    """Create a formatted @spec tag with all metadata"""
    description = get_requirement_description(req_id)
    doc_ref = get_spec_doc_ref(req_id)
    test_cases = get_test_cases(req_id)

    lines = [
        f"{comment_char} @spec:{req_id} - {description}",
        f"{comment_char} @ref:{doc_ref}",
    ]

    if test_cases != "N/A":
        lines.append(f"{comment_char} @test:{test_cases}")

    return '\n'.join(lines)


def add_tags_to_file(file_path: str, requirements: List[Tuple[str, str, str, str]]):
    """Add @spec tags to a source file"""
    if not os.path.exists(file_path):
        print(f"⚠️  File not found: {file_path}")
        return False

    # Read file
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    comment_char = get_comment_syntax(file_path)

    # Group requirements by line range
    tags_to_add = []
    for req_id, line_range, lang, status in requirements:
        # Skip if tag already exists
        if f"@spec:{req_id}" in content:
            print(f"   ⊙ Tag {req_id} already exists in {os.path.basename(file_path)}")
            continue

        tag = create_spec_tag(req_id, comment_char)
        tags_to_add.append((req_id, tag))

    if not tags_to_add:
        return False

    # Add tags at the top of the file (after imports for Python, after use statements for Rust)
    lines = content.split('\n')
    insert_line = 0

    # Find insertion point
    if file_path.endswith('.py'):
        # After imports and docstrings
        in_docstring = False
        for i, line in enumerate(lines):
            if '"""' in line or "'''" in line:
                in_docstring = not in_docstring
            if not in_docstring and not line.strip().startswith(('import ', 'from ', '#', '"""', "'''")) and line.strip():
                insert_line = i
                break
    elif file_path.endswith('.rs'):
        # After use statements
        for i, line in enumerate(lines):
            if line.strip() and not line.strip().startswith('use ') and not line.strip().startswith('//'):
                insert_line = i
                break
    elif file_path.endswith(('.ts', '.tsx')):
        # After imports
        for i, line in enumerate(lines):
            if line.strip() and not line.strip().startswith(('import ', 'export ', '//', '/*')):
                insert_line = i
                break

    # Insert all tags
    tags_text = '\n\n' + '\n\n'.join(tag for _, tag in tags_to_add) + '\n'
    lines.insert(insert_line, tags_text)

    # Write back
    new_content = '\n'.join(lines)
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(new_content)

    print(f"   ✅ Added {len(tags_to_add)} tags to {os.path.basename(file_path)}")
    for req_id, _ in tags_to_add:
        print(f"      • {req_id}")

    return True


def main():
    """Main entry point"""
    print("═" * 70)
    print("  Automatic @spec Tag Injection")
    print("═" * 70)
    print()

    # Parse traceability matrix
    requirements_to_code = parse_traceability_matrix()

    # Process each file
    print(f"\n📝 Processing {len(requirements_to_code)} files...")
    print()

    rust_count = 0
    python_count = 0
    ts_count = 0
    total_tags = 0

    for file_path, requirements in requirements_to_code.items():
        if 'rust-core-engine' in file_path:
            print(f"🦀 Rust: {os.path.basename(file_path)}")
            rust_count += 1
        elif 'python-ai-service' in file_path:
            print(f"🐍 Python: {os.path.basename(file_path)}")
            python_count += 1
        elif 'nextjs-ui-dashboard' in file_path:
            print(f"⚛️  TypeScript: {os.path.basename(file_path)}")
            ts_count += 1

        if add_tags_to_file(file_path, requirements):
            total_tags += len(requirements)

    print()
    print("═" * 70)
    print("  Summary")
    print("═" * 70)
    print(f"Rust files processed:       {rust_count}")
    print(f"Python files processed:     {python_count}")
    print(f"TypeScript files processed: {ts_count}")
    print(f"Total tags added:           {total_tags}")
    print()
    print("✅ @spec tag injection complete!")
    print()


if __name__ == '__main__':
    main()
