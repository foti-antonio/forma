"""Security regression checks for signing environment policy."""
import copy
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('policy', Path(__file__).with_name('release-policy.py'))
policy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(policy)


class SigningPolicyTests(unittest.TestCase):
    def setUp(self):
        self.environment = {'name': 'release-signing', 'protection_rules': [
            {'type': 'required_reviewers', 'prevent_self_review': True, 'reviewers': [{'type': 'User'}]}],
            'deployment_branch_policy': {'custom_branch_policies': True, 'protected_branches': False}}
        self.policies = {'total_count': 1, 'branch_policies': [{'type': 'tag', 'name': 'v0.1.0-beta.1'}]}

    def test_valid(self):
        policy.verify(self.environment, self.policies, 'v0.1.0-beta.1')

    def test_missing_or_self_review(self):
        for rules in ([], [{'type': 'required_reviewers', 'reviewers': [{}], 'prevent_self_review': False}]):
            env = copy.deepcopy(self.environment)
            env['protection_rules'] = rules
            with self.assertRaises(ValueError):
                policy.verify(env, self.policies, 'v0.1.0-beta.1')

    def test_unrestricted_environment(self):
        self.environment['deployment_branch_policy'] = None
        with self.assertRaises(ValueError):
            policy.verify(self.environment, self.policies, 'v0.1.0-beta.1')

    def test_branch_wildcard_wrong_tag_or_extra_policy(self):
        for rules in ([{'type': 'branch', 'name': 'v0.1.0-beta.1'}],
                      [{'type': 'tag', 'name': 'v*'}],
                      [{'type': 'tag', 'name': 'v0.1.0-beta.2'}],
                      self.policies['branch_policies'] * 2):
            with self.assertRaises(ValueError):
                policy.verify(self.environment, {'total_count': len(rules), 'branch_policies': rules}, 'v0.1.0-beta.1')
