"""
Test ML Library Performance
Verify performance is acceptable after updates

IMPORTANT: These tests require process isolation (--forked flag) to prevent
global state pollution between TensorFlow and PyTorch. Run with:
    pytest -c pytest_ml.ini tests/test_ml*.py

Regular pytest runs will skip these tests to avoid failures.
"""

import pytest
import time
import numpy as np
import os
import sys

# Skip these tests in regular pytest runs (without ML_TESTS environment variable)
# They must be run separately with: ML_TESTS=1 pytest -c pytest_ml.ini tests/test_ml*.py
pytestmark = pytest.mark.skipif(
    os.environ.get("ML_TESTS") != "1",
    reason="ML tests require process isolation. Run with: ML_TESTS=1 pytest -c pytest_ml.ini tests/test_ml*.py"
)


@pytest.mark.ml_isolated
@pytest.mark.forked
class TestPyTorchPerformance:
    """Test PyTorch inference and training performance"""

    def test_pytorch_inference_speed(self):
        """Verify PyTorch inference is fast enough"""
        import torch
        import torch.nn as nn

        # Create model
        model = nn.Sequential(
            nn.Linear(100, 200),
            nn.ReLU(),
            nn.Linear(200, 100),
            nn.ReLU(),
            nn.Linear(100, 10),
        )

        model.eval()

        # Create input
        x = torch.randn(32, 100)

        # Warmup
        with torch.no_grad():
            for _ in range(10):
                _ = model(x)

        # Measure
        start = time.time()
        with torch.no_grad():
            for _ in range(100):
                _ = model(x)
        elapsed = time.time() - start

        # Should be < 100ms per batch (very generous)
        avg_time = elapsed / 100
        assert avg_time < 0.1, f"Inference too slow: {avg_time:.4f}s per batch"

    def test_pytorch_training_speed(self):
        """Verify PyTorch training is acceptable"""
        import torch
        import torch.nn as nn
        import torch.optim as optim

        # Create model
        model = nn.Sequential(nn.Linear(50, 100), nn.ReLU(), nn.Linear(100, 1))

        optimizer = optim.Adam(model.parameters())
        criterion = nn.MSELoss()

        # Create data
        x = torch.randn(100, 50)
        y = torch.randn(100, 1)

        # Measure training time
        start = time.time()
        for _ in range(10):
            optimizer.zero_grad()
            output = model(x)
            loss = criterion(output, y)
            loss.backward()
            optimizer.step()
        elapsed = time.time() - start

        # Should complete 10 iterations in reasonable time
        assert elapsed < 5.0, f"Training too slow: {elapsed:.4f}s for 10 iterations"


@pytest.mark.ml_isolated
@pytest.mark.forked

class TestTensorFlowPerformance:
    """Test TensorFlow inference and training performance"""

    def test_tensorflow_inference_speed(self):
        """Verify TensorFlow inference is fast enough"""
        from tensorflow import keras
        import numpy as np

        # Create model
        model = keras.Sequential(
            [
                keras.layers.Dense(200, activation="relu", input_shape=(100,)),
                keras.layers.Dense(100, activation="relu"),
                keras.layers.Dense(10),
            ]
        )

        # Create input
        x = np.random.rand(32, 100).astype(np.float32)

        # Warmup
        for _ in range(10):
            _ = model.predict(x, verbose=0)

        # Measure
        start = time.time()
        for _ in range(100):
            _ = model.predict(x, verbose=0)
        elapsed = time.time() - start

        # Should be < 200ms per batch (very generous for TF)
        avg_time = elapsed / 100
        assert avg_time < 0.2, f"Inference too slow: {avg_time:.4f}s per batch"

    def test_tensorflow_training_speed(self):
        """Verify TensorFlow training is acceptable"""
        from tensorflow import keras
        import numpy as np

        # Create model
        model = keras.Sequential(
            [
                keras.layers.Dense(100, activation="relu", input_shape=(50,)),
                keras.layers.Dense(1),
            ]
        )

        model.compile(optimizer="adam", loss="mse")

        # Create data
        x = np.random.rand(100, 50).astype(np.float32)
        y = np.random.rand(100, 1).astype(np.float32)

        # Measure training time
        start = time.time()
        model.fit(x, y, epochs=10, verbose=0)
        elapsed = time.time() - start

        # Should complete in reasonable time
        assert elapsed < 10.0, f"Training too slow: {elapsed:.4f}s for 10 epochs"

@pytest.mark.ml_isolated
@pytest.mark.forked


class TestMemoryUsage:
    """Test memory usage is reasonable"""

    def test_pytorch_memory(self):
        """Test PyTorch doesn't use excessive memory"""
        import torch
        import torch.nn as nn

        # Create large model
        model = nn.Sequential(
            nn.Linear(1000, 2000),
            nn.ReLU(),
            nn.Linear(2000, 1000),
            nn.ReLU(),
            nn.Linear(1000, 100),
        )

        # Create large batch
        x = torch.randn(128, 1000)

        # Forward pass
        with torch.no_grad():
            output = model(x)

        # Should complete without OOM
        assert output.shape == (128, 100)

        # Cleanup
        del model
        del x
        del output
        torch.cuda.empty_cache() if torch.cuda.is_available() else None

    def test_tensorflow_memory(self):
        """Test TensorFlow doesn't use excessive memory"""
        from tensorflow import keras
        import numpy as np

        # Create large model
        model = keras.Sequential(
            [
                keras.layers.Dense(2000, activation="relu", input_shape=(1000,)),
                keras.layers.Dense(1000, activation="relu"),
                keras.layers.Dense(100),
            ]
        )

        # Create large batch
        x = np.random.rand(128, 1000).astype(np.float32)

        # Forward pass
        output = model.predict(x, verbose=0)

        # Should complete without OOM
        assert output.shape == (128, 100)

        # Cleanup
        del model
        del x
        del output
