"""Evidence schema rejects arbitrary event text and records only numeric counters."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from onboarding_metrics import OnboardingMetrics


class MetricsTests(unittest.TestCase):
    def test_local_evidence_has_only_bounded_stage_names_and_numeric_measurements(self):
        with patch('onboarding_metrics.time.monotonic', side_effect=[10, 12, 15]):
            metrics = OnboardingMetrics()
            metrics.actions = 3
            metrics.errors = 1
            metrics.recoveries = 1
            metrics.mark('authorized')
            with tempfile.TemporaryDirectory() as folder:
                file = Path(folder) / 'metrics.json'
                metrics.write(file)
                evidence = json.loads(file.read_text())
        self.assertEqual(evidence['stages'], {'authorized': {'elapsed_seconds': 2, 'actions': 3}})
        self.assertEqual(evidence['elapsed_seconds'], 5)
        self.assertEqual(evidence['result'], 'incomplete')
        self.assertEqual(evidence['errors'], 1)
        with self.assertRaises(ValueError):
            metrics.mark('private prompt or path')
        with self.assertRaises(ValueError):
            OnboardingMetrics('account@example.invalid')


if __name__ == '__main__':
    unittest.main()
