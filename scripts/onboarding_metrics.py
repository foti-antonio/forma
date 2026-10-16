"""Local test-only counters. Never record UI text, paths, prompts or provider output."""
import json
import time


class OnboardingMetrics:
    stages = {'setup', 'authorized', 'signed_in', 'repository', 'preview', 'completed', 'reviewed'}

    def __init__(self, provider='simulated'):
        if provider not in ('simulated', 'authenticated-codex'):
            raise ValueError('Unknown evidence kind')
        self.started = time.monotonic()
        self.provider = provider
        self.actions = 0
        self.errors = 0
        self.recoveries = 0
        self.marks = {}
        self.result = 'incomplete'

    def mark(self, stage):
        if stage not in self.stages:
            raise ValueError('Unknown measurement stage')
        self.marks[stage] = {'elapsed_seconds': round(time.monotonic() - self.started, 3),
                             'actions': self.actions}

    def write(self, path):
        path.write_text(json.dumps({'schema': 1, 'evidence': self.provider,
                                   'result': self.result, 'actions': self.actions,
                                   'errors': self.errors, 'recoveries': self.recoveries,
                                   'elapsed_seconds': round(time.monotonic() - self.started, 3),
                                   'stages': self.marks}, indent=2) + '\n')
