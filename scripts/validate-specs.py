#!/usr/bin/env python3
"""
Spec Validation Script for Bot Core
====================================

Validates that all specifications are in sync with code implementation.
This is CRITICAL for a finance project where mistakes = money loss.

Usage:
    python scripts/validate-specs.py
    python scripts/validate-specs.py --verbose
    python scripts/validate-specs.py --fix  # Auto-fix simple issues

Exit codes:
    0 = All validations passed
    1 = Validation errors found
    2 = Critical errors (missing files, parse errors)
"""

import os
import re
import sys
import json
import argparse
from pathlib import Path
from typing import Dict, List, Set, Tuple
from collections import defaultdict

# ANSI color codes
RED = '\033[91m'
GREEN = '\033[92m'
YELLOW = '\033[93m'
BLUE = '\033[94m'
MAGENTA = '\033[95m'
CYAN = '\033[96m'
BOLD = '\033[1m'
RESET = '\033[0m'

class SpecValidator:
    """Main validation class"""

    def __init__(self, root_dir: Path, verbose: bool = False, fix: bool = False):
        self.root_dir = root_dir
        self.verbose = verbose
        self.fix = fix
        self.errors = []
        self.warnings = []
        self.stats = defaultdict(int)

        # Paths
        self.specs_dir = root_dir / "specs"
        self.rust_src = root_dir / "rust-core-engine" / "src"
        self.python_src = root_dir / "python-ai-service"
        self.frontend_src = root_dir / "nextjs-ui-dashboard" / "src"

        # Collected data
        self.all_requirements: Set[str] = set()
        self.all_test_cases: Set[str] = set()
        self.spec_tags_in_code: Dict[str, List[Tuple[str, int]]] = defaultdict(list)
        self.requirement_mappings: Dict[str, Dict] = {}

    def log(self, message: str, level: str = "INFO"):
        """Log message with color"""
        colors = {
            "INFO": CYAN,
            "SUCCESS": GREEN,
            "WARNING": YELLOW,
            "ERROR": RED,
            "CRITICAL": BOLD + RED
        }
        color = colors.get(level, "")
        print(f"{color}{level}{RESET}: {message}")

    def error(self, message: str):
        """Log error and add to errors list"""
        self.errors.append(message)
        self.log(message, "ERROR")

    def warning(self, message: str):
        """Log warning and add to warnings list"""
        self.warnings.append(message)
        self.log(message, "WARNING")

    def success(self, message: str):
        """Log success message"""
        self.log(message, "SUCCESS")

    # ========================================================================
    # PHASE 1: Collect all requirements from specs
    # ========================================================================

    def collect_requirements(self):
        """Scan all FR-*.md files to collect requirement IDs"""
        self.log("Collecting all requirements from specs...", "INFO")

        fr_dir = self.specs_dir / "01-requirements" / "1.1-functional-requirements"
        if not fr_dir.exists():
            self.error(f"Functional requirements directory not found: {fr_dir}")
            return

        fr_pattern = re.compile(r'###\s+(FR-[A-Z]+-\d+)[:|\s]')

        for fr_file in fr_dir.glob("FR-*.md"):
            if not fr_file.is_file():
                continue

            try:
                content = fr_file.read_text(encoding='utf-8')
                matches = fr_pattern.findall(content)

                for req_id in matches:
                    self.all_requirements.add(req_id)
                    self.stats['requirements_found'] += 1

                    if self.verbose:
                        self.log(f"  Found {req_id} in {fr_file.name}", "INFO")

            except Exception as e:
                self.error(f"Failed to read {fr_file}: {e}")

        self.success(f"Collected {len(self.all_requirements)} requirements from {len(list(fr_dir.glob('FR-*.md')))} files")

    # ========================================================================
    # PHASE 2: Collect all test cases from specs
    # ========================================================================

    def collect_test_cases(self):
        """Scan all TC-*.md files to collect test case IDs"""
        self.log("Collecting all test cases from specs...", "INFO")

        tc_dir = self.specs_dir / "03-testing" / "3.2-test-cases"
        if not tc_dir.exists():
            self.error(f"Test cases directory not found: {tc_dir}")
            return

        tc_pattern = re.compile(r'###\s+(TC-[A-Z]+-\d+)[:|\s]')

        for tc_file in tc_dir.glob("TC-*.md"):
            if not tc_file.is_file():
                continue

            try:
                content = tc_file.read_text(encoding='utf-8')
                matches = tc_pattern.findall(content)

                for tc_id in matches:
                    self.all_test_cases.add(tc_id)
                    self.stats['test_cases_found'] += 1

                    if self.verbose:
                        self.log(f"  Found {tc_id} in {tc_file.name}", "INFO")

            except Exception as e:
                self.error(f"Failed to read {tc_file}: {e}")

        self.success(f"Collected {len(self.all_test_cases)} test cases from {len(list(tc_dir.glob('TC-*.md')))} files")

    # ========================================================================
    # PHASE 3: Scan code for @spec tags
    # ========================================================================

    def scan_code_for_spec_tags(self):
        """Scan all source code files for @spec tags"""
        self.log("Scanning code for @spec tags...", "INFO")

        # Patterns for different languages
        spec_patterns = {
            'rust': re.compile(r'//\s*@spec:(FR-[A-Z]+-\d+)'),
            'python': re.compile(r'#\s*@spec:(FR-[A-Z]+-\d+)'),
            'typescript': re.compile(r'//\s*@spec:(FR-[A-Z]+-\d+)')
        }

        # Scan Rust files
        self._scan_directory(self.rust_src, ['.rs'], spec_patterns['rust'], 'Rust')

        # Scan Python files
        self._scan_directory(self.python_src, ['.py'], spec_patterns['python'], 'Python')

        # Scan TypeScript files
        self._scan_directory(self.frontend_src, ['.ts', '.tsx'], spec_patterns['typescript'], 'TypeScript')

        total_tags = sum(len(v) for v in self.spec_tags_in_code.values())
        self.success(f"Found {total_tags} @spec tags referencing {len(self.spec_tags_in_code)} unique requirements")

    def _scan_directory(self, directory: Path, extensions: List[str], pattern: re.Pattern, lang: str):
        """Helper to scan a directory for spec tags"""
        if not directory.exists():
            self.warning(f"{lang} directory not found: {directory}")
            return

        for ext in extensions:
            for file_path in directory.rglob(f"*{ext}"):
                if not file_path.is_file():
                    continue

                # Skip test files and node_modules
                if 'test' in str(file_path).lower() or 'node_modules' in str(file_path):
                    continue

                try:
                    content = file_path.read_text(encoding='utf-8')

                    for line_num, line in enumerate(content.splitlines(), 1):
                        matches = pattern.findall(line)
                        for req_id in matches:
                            rel_path = file_path.relative_to(self.root_dir)
                            self.spec_tags_in_code[req_id].append((str(rel_path), line_num))
                            self.stats['spec_tags_found'] += 1

                            if self.verbose:
                                self.log(f"  {req_id} @ {rel_path}:{line_num}", "INFO")

                except Exception as e:
                    self.error(f"Failed to read {file_path}: {e}")

    # ========================================================================
    # PHASE 4: Load traceability matrix
    # ========================================================================

    def load_traceability_matrix(self):
        """Parse TRACEABILITY_MATRIX.md to extract mappings"""
        self.log("Loading traceability matrix...", "INFO")

        matrix_file = self.specs_dir / "TRACEABILITY_MATRIX.md"
        if not matrix_file.exists():
            self.error(f"TRACEABILITY_MATRIX.md not found at {matrix_file}")
            return

        try:
            content = matrix_file.read_text(encoding='utf-8')

            # Parse table rows: | FR-XXX-YYY | Description | Design Docs | Test Cases | Status |
            table_pattern = re.compile(
                r'\|\s*(FR-[A-Z]+-\d+)\s*\|\s*([^|]+)\s*\|\s*([^|]+)\s*\|\s*([^|]+)\s*\|\s*([^|]+)\s*\|'
            )

            for match in table_pattern.finditer(content):
                req_id = match.group(1).strip()
                description = match.group(2).strip()
                design_docs = match.group(3).strip()
                test_cases = match.group(4).strip()
                status = match.group(5).strip()

                self.requirement_mappings[req_id] = {
                    'description': description,
                    'design_docs': [d.strip() for d in design_docs.split(',') if d.strip()],
                    'test_cases': [t.strip() for t in test_cases.split(',') if t.strip()],
                    'status': status
                }

                if self.verbose:
                    self.log(f"  Mapped {req_id}: {len(self.requirement_mappings[req_id]['test_cases'])} test cases", "INFO")

            self.success(f"Loaded {len(self.requirement_mappings)} requirement mappings from traceability matrix")

        except Exception as e:
            self.error(f"Failed to parse traceability matrix: {e}")

    # ========================================================================
    # PHASE 5: Validation checks
    # ========================================================================

    def validate_spec_tags_exist(self):
        """Check that all @spec tags reference valid requirements"""
        self.log("\n[CHECK 1] Validating @spec tags reference existing requirements...", "INFO")

        invalid_tags = []
        for req_id, locations in self.spec_tags_in_code.items():
            if req_id not in self.all_requirements:
                invalid_tags.append(req_id)
                for file_path, line_num in locations:
                    self.error(f"  Invalid @spec tag {req_id} at {file_path}:{line_num} - requirement does not exist")

        if not invalid_tags:
            self.success(f"✓ All {self.stats['spec_tags_found']} @spec tags reference valid requirements")
            self.stats['checks_passed'] += 1
        else:
            self.error(f"✗ Found {len(invalid_tags)} invalid @spec tags")
            self.stats['checks_failed'] += 1

    def validate_requirements_have_code(self):
        """Check that all requirements have corresponding @spec tags in code"""
        self.log("\n[CHECK 2] Validating all requirements have code implementation...", "INFO")

        missing_code = []
        for req_id in self.all_requirements:
            if req_id not in self.spec_tags_in_code:
                missing_code.append(req_id)
                self.warning(f"  {req_id} has no @spec tags in code (might not be implemented yet)")

        coverage = ((len(self.all_requirements) - len(missing_code)) / len(self.all_requirements)) * 100

        if coverage == 100:
            self.success(f"✓ 100% requirement coverage - all {len(self.all_requirements)} requirements have code")
            self.stats['checks_passed'] += 1
        elif coverage >= 90:
            self.warning(f"⚠ {coverage:.1f}% coverage - {len(missing_code)} requirements missing code")
            self.stats['checks_warning'] += 1
        else:
            self.error(f"✗ Only {coverage:.1f}% coverage - {len(missing_code)} requirements missing code")
            self.stats['checks_failed'] += 1

    def validate_traceability_matrix_complete(self):
        """Check that traceability matrix includes all requirements"""
        self.log("\n[CHECK 3] Validating traceability matrix completeness...", "INFO")

        missing_from_matrix = []
        for req_id in self.all_requirements:
            if req_id not in self.requirement_mappings:
                missing_from_matrix.append(req_id)
                self.warning(f"  {req_id} exists in specs but not in TRACEABILITY_MATRIX.md")

        if not missing_from_matrix:
            self.success(f"✓ Traceability matrix is complete - all {len(self.all_requirements)} requirements mapped")
            self.stats['checks_passed'] += 1
        else:
            self.error(f"✗ {len(missing_from_matrix)} requirements missing from traceability matrix")
            self.stats['checks_failed'] += 1

    def validate_test_cases_mapped(self):
        """Check that all test cases in matrix exist in spec files"""
        self.log("\n[CHECK 4] Validating test case references...", "INFO")

        invalid_test_refs = []
        for req_id, mapping in self.requirement_mappings.items():
            for tc_id in mapping['test_cases']:
                # Clean up test case ID (remove extra text)
                tc_clean = tc_id.split()[0] if tc_id else ""

                if tc_clean and tc_clean not in self.all_test_cases:
                    invalid_test_refs.append(f"{req_id} -> {tc_clean}")
                    self.warning(f"  {req_id} references non-existent test case {tc_clean}")

        if not invalid_test_refs:
            self.success(f"✓ All test case references are valid")
            self.stats['checks_passed'] += 1
        else:
            self.warning(f"⚠ Found {len(invalid_test_refs)} invalid test case references")
            self.stats['checks_warning'] += 1

    def validate_design_docs_exist(self):
        """Check that design documents referenced in matrix exist"""
        self.log("\n[CHECK 5] Validating design document references...", "INFO")

        design_dir = self.specs_dir / "02-design"
        missing_docs = []

        for req_id, mapping in self.requirement_mappings.items():
            for doc_name in mapping['design_docs']:
                # Extract just the filename (e.g., "COMP-RUST-TRADING.md")
                doc_clean = doc_name.split()[0] if doc_name else ""

                if not doc_clean or doc_clean == "-":
                    continue

                # Search for this doc in design directory
                found = False
                for subdir in design_dir.rglob("*"):
                    if subdir.is_dir():
                        doc_path = subdir / doc_clean
                        if doc_path.exists():
                            found = True
                            break

                if not found:
                    missing_docs.append(f"{req_id} -> {doc_clean}")
                    self.warning(f"  {req_id} references missing design doc {doc_clean}")

        if not missing_docs:
            self.success(f"✓ All design document references are valid")
            self.stats['checks_passed'] += 1
        else:
            self.warning(f"⚠ Found {len(missing_docs)} missing design document references")
            self.stats['checks_warning'] += 1

    # ========================================================================
    # Main validation flow
    # ========================================================================

    def run(self) -> int:
        """Run all validation checks"""
        print(f"\n{BOLD}{CYAN}╔══════════════════════════════════════════════════════════════╗{RESET}")
        print(f"{BOLD}{CYAN}║         Bot Core Specification Validation Script          ║{RESET}")
        print(f"{BOLD}{CYAN}║              Finance Project - CRITICAL CHECKS             ║{RESET}")
        print(f"{BOLD}{CYAN}╚══════════════════════════════════════════════════════════════╝{RESET}\n")

        # Collection phase
        self.collect_requirements()
        self.collect_test_cases()
        self.scan_code_for_spec_tags()
        self.load_traceability_matrix()

        # Validation phase
        self.validate_spec_tags_exist()
        self.validate_requirements_have_code()
        self.validate_traceability_matrix_complete()
        self.validate_test_cases_mapped()
        self.validate_design_docs_exist()

        # Summary
        self.print_summary()

        # Exit code
        if self.errors:
            return 1
        return 0

    def print_summary(self):
        """Print validation summary"""
        print(f"\n{BOLD}{CYAN}{'='*80}{RESET}")
        print(f"{BOLD}{CYAN}VALIDATION SUMMARY{RESET}")
        print(f"{BOLD}{CYAN}{'='*80}{RESET}\n")

        print(f"{BOLD}Collection Phase:{RESET}")
        print(f"  Requirements found:    {GREEN}{len(self.all_requirements)}{RESET}")
        print(f"  Test cases found:      {GREEN}{len(self.all_test_cases)}{RESET}")
        print(f"  @spec tags found:      {GREEN}{self.stats['spec_tags_found']}{RESET}")
        print(f"  Traceability mappings: {GREEN}{len(self.requirement_mappings)}{RESET}")

        print(f"\n{BOLD}Validation Results:{RESET}")
        print(f"  {GREEN}✓ Checks passed:  {self.stats['checks_passed']}{RESET}")
        print(f"  {YELLOW}⚠ Checks warning: {self.stats.get('checks_warning', 0)}{RESET}")
        print(f"  {RED}✗ Checks failed:  {self.stats['checks_failed']}{RESET}")

        if self.errors:
            print(f"\n{RED}{BOLD}ERRORS: {len(self.errors)}{RESET}")
            for error in self.errors[:10]:  # Show first 10
                print(f"  {RED}• {error}{RESET}")
            if len(self.errors) > 10:
                print(f"  {RED}... and {len(self.errors) - 10} more errors{RESET}")

        if self.warnings:
            print(f"\n{YELLOW}{BOLD}WARNINGS: {len(self.warnings)}{RESET}")
            for warning in self.warnings[:10]:  # Show first 10
                print(f"  {YELLOW}• {warning}{RESET}")
            if len(self.warnings) > 10:
                print(f"  {YELLOW}... and {len(self.warnings) - 10} more warnings{RESET}")

        # Final verdict
        print(f"\n{BOLD}{CYAN}{'='*80}{RESET}")
        if not self.errors:
            print(f"{GREEN}{BOLD}✓ ALL VALIDATIONS PASSED - SPECS ARE IN SYNC WITH CODE{RESET}")
            print(f"{GREEN}  Finance project quality: EXCELLENT ⭐⭐⭐⭐⭐{RESET}")
        elif self.stats['checks_failed'] <= 2:
            print(f"{YELLOW}{BOLD}⚠ MINOR ISSUES FOUND - REQUIRES ATTENTION{RESET}")
            print(f"{YELLOW}  Finance project quality: GOOD (needs minor fixes){RESET}")
        else:
            print(f"{RED}{BOLD}✗ CRITICAL ISSUES FOUND - IMMEDIATE ACTION REQUIRED{RESET}")
            print(f"{RED}  Finance project quality: NEEDS IMPROVEMENT{RESET}")
        print(f"{BOLD}{CYAN}{'='*80}{RESET}\n")


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(
        description="Validate Bot Core specifications against code implementation",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python scripts/validate-specs.py                    # Run validation
  python scripts/validate-specs.py --verbose          # Verbose output
  python scripts/validate-specs.py --fix              # Auto-fix issues (future)
        """
    )
    parser.add_argument('-v', '--verbose', action='store_true', help='Enable verbose output')
    parser.add_argument('--fix', action='store_true', help='Auto-fix simple issues (not implemented yet)')

    args = parser.parse_args()

    # Find project root
    script_dir = Path(__file__).parent
    root_dir = script_dir.parent

    if not (root_dir / "specs").exists():
        print(f"{RED}ERROR: specs directory not found at {root_dir / 'specs'}{RESET}")
        print(f"{RED}Please run this script from the project root.{RESET}")
        return 2

    # Run validation
    validator = SpecValidator(root_dir, verbose=args.verbose, fix=args.fix)
    exit_code = validator.run()

    sys.exit(exit_code)


if __name__ == "__main__":
    main()
