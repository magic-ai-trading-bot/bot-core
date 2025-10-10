#!/usr/bin/env python3
"""
Validate @spec tags in source code files
Ensures all code has traceability tags linking to specifications
"""

import os
import re
from pathlib import Path
from typing import Dict, List, Set, Tuple
from collections import defaultdict

# Project root
PROJECT_ROOT = Path(__file__).parent.parent

# Colors for terminal output
class Colors:
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    RED = '\033[0;31m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'  # No Color


def find_spec_tags_in_file(file_path: Path) -> List[Tuple[str, int]]:
    """Find all @spec tags in a file and return (spec_id, line_number) tuples"""
    tags = []
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            for line_num, line in enumerate(f, 1):
                # Match @spec:FR-XXX-YYY or similar pattern
                matches = re.findall(r'@spec:([A-Z]+-[A-Z]+-\d+|[A-Z]+-[A-Z]+\s+\([^)]+\))', line)
                for match in matches:
                    tags.append((match.strip(), line_num))
    except Exception as e:
        print(f"{Colors.RED}Error reading {file_path}: {e}{Colors.NC}")
    return tags


def scan_codebase() -> Dict[str, List[Tuple[str, int]]]:
    """Scan all source files for @spec tags"""
    results = defaultdict(list)

    # Rust files
    rust_dir = PROJECT_ROOT / "rust-core-engine" / "src"
    if rust_dir.exists():
        for rust_file in rust_dir.rglob("*.rs"):
            tags = find_spec_tags_in_file(rust_file)
            if tags:
                rel_path = rust_file.relative_to(PROJECT_ROOT)
                results[str(rel_path)].extend(tags)

    # Python files
    python_dir = PROJECT_ROOT / "python-ai-service"
    if python_dir.exists():
        for py_file in python_dir.rglob("*.py"):
            # Skip venv and cache
            if 'venv' in str(py_file) or '__pycache__' in str(py_file):
                continue
            tags = find_spec_tags_in_file(py_file)
            if tags:
                rel_path = py_file.relative_to(PROJECT_ROOT)
                results[str(rel_path)].extend(tags)

    # TypeScript files
    frontend_dir = PROJECT_ROOT / "nextjs-ui-dashboard" / "src"
    if frontend_dir.exists():
        for ts_file in frontend_dir.rglob("*.ts*"):
            # Skip node_modules
            if 'node_modules' in str(ts_file):
                continue
            tags = find_spec_tags_in_file(ts_file)
            if tags:
                rel_path = ts_file.relative_to(PROJECT_ROOT)
                results[str(rel_path)].extend(tags)

    return dict(results)


def validate_spec_tag_format(spec_id: str) -> bool:
    """Validate that spec tag follows correct format"""
    patterns = [
        r'^FR-[A-Z]+-\d+$',        # FR-AUTH-001
        r'^NFR-[A-Z]+-\d+$',       # NFR-SECURITY-001
        r'^US-[A-Z]+-\d+$',        # US-TRADER-001
        r'^TC-[A-Z]+-\d+$',        # TC-AUTH-001
    ]
    for pattern in patterns:
        if re.match(pattern, spec_id):
            return True
    # Also accept special formats like "FR-AI-005 (Frontend)"
    if '(' in spec_id and ')' in spec_id:
        return True
    return False


def check_missing_tags() -> List[str]:
    """Check for files that should have tags but don't"""
    missing = []

    # Check major module files
    important_files = [
        'rust-core-engine/src/auth/jwt.rs',
        'rust-core-engine/src/trading/engine.rs',
        'python-ai-service/main.py',
        'nextjs-ui-dashboard/src/hooks/useWebSocket.ts',
    ]

    for file_path in important_files:
        full_path = PROJECT_ROOT / file_path
        if full_path.exists():
            tags = find_spec_tags_in_file(full_path)
            if not tags:
                missing.append(file_path)

    return missing


def main():
    """Main validation entry point"""
    print("=" * 70)
    print(f"{Colors.BLUE}  @spec Tag Validation Report{Colors.NC}")
    print("=" * 70)
    print()

    # Scan codebase
    print(f"{Colors.BLUE}Scanning codebase for @spec tags...{Colors.NC}")
    tagged_files = scan_codebase()

    # Statistics
    total_files = len(tagged_files)
    total_tags = sum(len(tags) for tags in tagged_files.values())

    # Count by language
    rust_files = sum(1 for f in tagged_files.keys() if 'rust-core-engine' in f)
    python_files = sum(1 for f in tagged_files.keys() if 'python-ai-service' in f)
    ts_files = sum(1 for f in tagged_files.keys() if 'nextjs-ui-dashboard' in f)

    print()
    print(f"{Colors.GREEN}✓ Found {total_tags} @spec tags in {total_files} files{Colors.NC}")
    print()
    print(f"  Rust files:       {rust_files}")
    print(f"  Python files:     {python_files}")
    print(f"  TypeScript files: {ts_files}")
    print()

    # Validate tag formats
    print(f"{Colors.BLUE}Validating tag formats...{Colors.NC}")
    invalid_tags = []
    for file_path, tags in tagged_files.items():
        for spec_id, line_num in tags:
            if not validate_spec_tag_format(spec_id):
                invalid_tags.append((file_path, spec_id, line_num))

    if invalid_tags:
        print(f"{Colors.YELLOW}⚠ Found {len(invalid_tags)} invalid tag formats:{Colors.NC}")
        for file_path, spec_id, line_num in invalid_tags[:10]:  # Show first 10
            print(f"  {file_path}:{line_num} - {spec_id}")
        if len(invalid_tags) > 10:
            print(f"  ... and {len(invalid_tags) - 10} more")
    else:
        print(f"{Colors.GREEN}✓ All tags follow correct format{Colors.NC}")
    print()

    # Check for missing tags
    print(f"{Colors.BLUE}Checking for missing tags in important files...{Colors.NC}")
    missing = check_missing_tags()
    if missing:
        print(f"{Colors.YELLOW}⚠ Found {len(missing)} important files without tags:{Colors.NC}")
        for file_path in missing:
            print(f"  {file_path}")
    else:
        print(f"{Colors.GREEN}✓ All important files have @spec tags{Colors.NC}")
    print()

    # Show tag distribution by spec category
    print(f"{Colors.BLUE}Tag distribution by category:{Colors.NC}")
    categories = defaultdict(int)
    for tags in tagged_files.values():
        for spec_id, _ in tags:
            # Extract category (e.g., FR-AUTH, FR-TRADING)
            if '-' in spec_id:
                parts = spec_id.split('-')
                if len(parts) >= 2:
                    category = f"{parts[0]}-{parts[1]}"
                    categories[category] += 1

    for category, count in sorted(categories.items(), key=lambda x: x[1], reverse=True):
        print(f"  {category:20s} {count:3d} tags")
    print()

    # Detailed file listing
    if len(tagged_files) <= 50:  # Only show if manageable
        print(f"{Colors.BLUE}Tagged files:{Colors.NC}")
        for file_path, tags in sorted(tagged_files.items()):
            tag_ids = [spec_id for spec_id, _ in tags]
            print(f"  {file_path}")
            print(f"    Tags: {', '.join(tag_ids)}")
        print()

    # Summary
    print("=" * 70)
    print(f"{Colors.BLUE}  Summary{Colors.NC}")
    print("=" * 70)
    print(f"Total files with tags:  {total_files}")
    print(f"Total @spec tags:       {total_tags}")
    print(f"Invalid formats:        {len(invalid_tags)}")
    print(f"Missing important tags: {len(missing)}")
    print()

    # Overall status
    if invalid_tags or missing:
        print(f"{Colors.YELLOW}⚠ Validation completed with warnings{Colors.NC}")
        return 1
    else:
        print(f"{Colors.GREEN}✓ All validations passed!{Colors.NC}")
        return 0


if __name__ == '__main__':
    exit(main())
