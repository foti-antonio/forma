#!/usr/bin/env python3
"""Fail closed unless signing has independent review and exact-tag isolation."""
import json
from pathlib import Path
import sys


def verify(environment, policies, tag):
    if environment.get('name') != 'release-signing':
        raise ValueError('Wrong signing environment')
    if not any(r.get('type') == 'required_reviewers' and r.get('prevent_self_review') is True
               and r.get('reviewers') for r in environment.get('protection_rules', [])):
        raise ValueError('Independent required reviewer protection is missing')
    policy = environment.get('deployment_branch_policy') or {}
    if policy.get('custom_branch_policies') is not True or policy.get('protected_branches') is not False:
        raise ValueError('Custom tag deployment policy required')
    rules = policies.get('branch_policies', [])
    if policies.get('total_count') != 1 or len(rules) != 1 or rules[0].get('type') != 'tag' or rules[0].get('name') != tag:
        raise ValueError('Allow only the exact approved tag; no branch or wildcard policies')


if __name__ == '__main__':
    verify(json.loads(Path(sys.argv[1]).read_text()), json.loads(Path(sys.argv[2]).read_text()), sys.argv[3])
    print('PASS: independent reviewer and exact-tag signing deployment policy')
