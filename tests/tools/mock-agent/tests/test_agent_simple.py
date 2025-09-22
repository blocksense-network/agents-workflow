#!/usr/bin/env python3
"""
Simple test suite for the mock coding agent that doesn't require pytest.

This script runs tests and provides a simple way to verify
the agent functionality without requiring external test frameworks.
"""

import os
import json
import tempfile
import shutil
import subprocess
import sys
import pexpect
from pathlib import Path


def log(message):
    """Simple logging function."""
    print(f"[TEST] {message}")


def assert_true(condition, message="Assertion failed"):
    """Simple assertion helper."""
    if not condition:
        log(f"FAIL: {message}")
        raise AssertionError(message)
    log(f"PASS: {message}")


class TestRunner:
    """Simple test runner for the mock coding agent."""
    
    def __init__(self):
        self.project_root = Path(__file__).parent.parent
        self.tests_passed = 0
        self.tests_failed = 0
    
    def create_temp_workspace(self):
        """Create a temporary workspace directory."""
        return tempfile.mkdtemp(prefix="mock_agent_test_")
    
    def create_temp_codex_home(self):
        """Create a temporary codex home directory."""
        return tempfile.mkdtemp(prefix="mock_agent_codex_")
    
    def cleanup(self, *paths):
        """Clean up temporary directories."""
        for path in paths:
            if os.path.exists(path):
                shutil.rmtree(path)
    
    def run_test(self, test_name, test_func):
        """Run a single test and track results."""
        log(f"Running test: {test_name}")
        try:
            test_func()
            self.tests_passed += 1
            log(f"✓ {test_name} PASSED")
        except Exception as e:
            self.tests_failed += 1
            log(f"✗ {test_name} FAILED: {e}")
    
    def test_hello_scenario_file_creation(self):
        """Test that running the hello scenario creates the expected file."""
        workspace = self.create_temp_workspace()
        codex_home = self.create_temp_codex_home()
        
        try:
            scenario_path = self.project_root / "examples" / "hello_scenario.json"
            
            # Run the agent
            result = subprocess.run([
                sys.executable, "-m", "src.cli", "run",
                "--scenario", str(scenario_path),
                "--workspace", workspace,
                "--codex-home", codex_home
            ], cwd=self.project_root, capture_output=True, text=True)
            
            # Verify the command succeeded
            assert_true(result.returncode == 0, f"Command failed with code {result.returncode}: {result.stderr}")
            
            # Verify hello.py was created
            hello_file = Path(workspace) / "hello.py"
            assert_true(hello_file.exists(), "hello.py was not created")
            
            # Verify the content is correct
            content = hello_file.read_text()
            assert_true("print('Hello, World!')" in content, f"Unexpected content: {content}")
            
        finally:
            self.cleanup(workspace, codex_home)
    
    def test_hello_scenario_terminal_output(self):
        """Test that the agent produces expected terminal output."""
        workspace = self.create_temp_workspace()
        codex_home = self.create_temp_codex_home()
        
        try:
            scenario_path = self.project_root / "examples" / "hello_scenario.json"
            
            # Use pexpect to capture live output
            proc = pexpect.spawn(
                sys.executable, ["-m", "src.cli", "run",
                              "--scenario", str(scenario_path),
                              "--workspace", workspace,
                              "--codex-home", codex_home],
                cwd=str(self.project_root),
                timeout=30
            )
            
            try:
                # Expect user input trace
                proc.expect(r"\[user\] Please create hello\.py that prints Hello, World!")
                
                # Expect thinking trace  
                proc.expect(r"\[thinking\] I'll create hello\.py with a print statement\.")
                
                # Expect tool call trace (write_file)
                proc.expect(r"\[tool\] write_file")
                
                # Expect tool result trace
                proc.expect(r"write_file -> ok")
                
                # Expect assistant response
                proc.expect(r"\[assistant\] Created hello\.py\. Run: python hello\.py")
                
                # Wait for completion
                proc.expect(pexpect.EOF)
                
                # Wait for the process to actually terminate
                proc.wait()
                
                # Check exit status - note that pexpect sometimes reports None for successful processes
                exit_status = proc.exitstatus
                if exit_status is None:
                    # If exitstatus is None, check if process terminated normally
                    assert_true(proc.isalive() == False, "Process should have terminated")
                    # Also verify the expected file was created as additional validation
                    hello_file = Path(workspace) / "hello.py"
                    assert_true(hello_file.exists(), "hello.py should have been created")
                else:
                    assert_true(exit_status == 0, f"Process failed with exit code {exit_status}")
                
            finally:
                proc.close()
                
        finally:
            self.cleanup(workspace, codex_home)
    
    def test_demo_scenario(self):
        """Test the built-in demo scenario."""
        workspace = self.create_temp_workspace()
        codex_home = self.create_temp_codex_home()
        
        try:
            result = subprocess.run([
                sys.executable, "-m", "src.cli", "demo",
                "--workspace", workspace,
                "--codex-home", codex_home
            ], cwd=self.project_root, capture_output=True, text=True)
            
            # Verify the command succeeded
            assert_true(result.returncode == 0, f"Demo command failed with code {result.returncode}: {result.stderr}")
            
            # Verify the demo scenario file was created
            demo_scenario = Path(workspace) / "_demo_scenario.json"
            assert_true(demo_scenario.exists(), "Demo scenario file was not created")
            
            # Verify it's valid JSON
            with open(demo_scenario) as f:
                scenario_data = json.load(f)
            
            assert_true("meta" in scenario_data, "Demo scenario missing meta section")
            assert_true("turns" in scenario_data, "Demo scenario missing turns section")
            
        finally:
            self.cleanup(workspace, codex_home)
    
    def test_rollout_file_creation(self):
        """Test that rollout files are created in the correct location."""
        workspace = self.create_temp_workspace()
        codex_home = self.create_temp_codex_home()
        
        try:
            scenario_path = self.project_root / "examples" / "hello_scenario.json"
            
            result = subprocess.run([
                sys.executable, "-m", "src.cli", "run",
                "--scenario", str(scenario_path),
                "--workspace", workspace,
                "--codex-home", codex_home
            ], cwd=self.project_root, capture_output=True, text=True)
            
            assert_true(result.returncode == 0, f"Command failed with code {result.returncode}: {result.stderr}")
            
            # Check that rollout files were created
            sessions_dir = Path(codex_home) / "sessions"
            assert_true(sessions_dir.exists(), "Sessions directory was not created")
            
            # Find rollout files (they have date-based subdirectories)
            rollout_files = list(sessions_dir.rglob("rollout-*.jsonl"))
            assert_true(len(rollout_files) > 0, "No rollout files were created")
            
            # Verify the rollout file contains valid JSONL
            rollout_file = rollout_files[0]
            with open(rollout_file) as f:
                lines = f.readlines()
            
            assert_true(len(lines) > 0, "Rollout file is empty")
            
            # Verify each line is valid JSON
            for line in lines:
                line = line.strip()
                if line:
                    json.loads(line)  # This will raise if invalid JSON
                    
        finally:
            self.cleanup(workspace, codex_home)
    
    def test_cli_help(self):
        """Test that CLI help commands work."""
        result = subprocess.run([
            sys.executable, "-m", "src.cli", "--help"
        ], cwd=self.project_root, capture_output=True, text=True)
        
        assert_true(result.returncode == 0, f"Help command failed with code {result.returncode}: {result.stderr}")
        assert_true("Mock Coding Agent" in result.stdout, "Help text missing expected content")
        assert_true("run" in result.stdout, "Help missing 'run' command")
        assert_true("demo" in result.stdout, "Help missing 'demo' command")
        assert_true("server" in result.stdout, "Help missing 'server' command")
    
    def test_file_operations(self):
        """Test various file operations in scenarios."""
        workspace = self.create_temp_workspace()
        codex_home = self.create_temp_codex_home()
        
        try:
            # Create a custom scenario that tests multiple file operations
            custom_scenario = {
                "meta": {
                    "instructions": "Test file operations"
                },
                "turns": [
                    {"user": "Create and modify files for testing"},
                    {"tool": {"name": "write_file", "args": {"path": "test.txt", "text": "Initial content\n"}}},
                    {"tool": {"name": "read_file", "args": {"path": "test.txt"}}},
                    {"tool": {"name": "append_file", "args": {"path": "test.txt", "text": "Appended content\n"}}},
                    {"tool": {"name": "read_file", "args": {"path": "test.txt"}}},
                    {"assistant": "Files created and modified successfully."}
                ]
            }
            
            # Write the scenario to a temporary file
            scenario_file = Path(workspace) / "test_scenario.json"
            with open(scenario_file, "w") as f:
                json.dump(custom_scenario, f)
            
            # Run the scenario
            result = subprocess.run([
                sys.executable, "-m", "src.cli", "run",
                "--scenario", str(scenario_file),
                "--workspace", workspace,
                "--codex-home", codex_home
            ], cwd=self.project_root, capture_output=True, text=True)
            
            assert_true(result.returncode == 0, f"Command failed with code {result.returncode}: {result.stderr}")
            
            # Verify the file was created and has the expected content
            test_file = Path(workspace) / "test.txt"
            assert_true(test_file.exists(), "test.txt was not created")
            
            content = test_file.read_text()
            assert_true("Initial content" in content, "Initial content not found")
            assert_true("Appended content" in content, "Appended content not found")
            
        finally:
            self.cleanup(workspace, codex_home)
    
    def run_all_tests(self):
        """Run all tests and report results."""
        log("Starting mock agent test suite")
        log(f"Project root: {self.project_root}")
        
        # List of all test methods
        tests = [
            ("CLI Help", self.test_cli_help),
            ("Hello Scenario File Creation", self.test_hello_scenario_file_creation),
            ("Hello Scenario Terminal Output", self.test_hello_scenario_terminal_output),
            ("Demo Scenario", self.test_demo_scenario),
            ("Rollout File Creation", self.test_rollout_file_creation),
            ("File Operations", self.test_file_operations),
        ]
        
        # Run all tests
        for test_name, test_func in tests:
            self.run_test(test_name, test_func)
        
        # Report results
        total_tests = self.tests_passed + self.tests_failed
        log(f"\nTest Results:")
        log(f"  Total tests: {total_tests}")
        log(f"  Passed: {self.tests_passed}")
        log(f"  Failed: {self.tests_failed}")
        
        if self.tests_failed == 0:
            log("🎉 All tests passed!")
            return 0
        else:
            log(f"❌ {self.tests_failed} test(s) failed")
            return 1


def main():
    """Main entry point."""
    runner = TestRunner()
    return runner.run_all_tests()


if __name__ == "__main__":
    sys.exit(main())