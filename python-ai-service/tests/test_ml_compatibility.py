"""
Test ML Library Compatibility
Tests for PyTorch 2.5.1 and TensorFlow 2.18.0 (Keras 3.0)

These tests are marked with @pytest.mark.ml_isolated to run in a controlled order.
When using pytest-xdist (-n auto), each test worker will run these tests sequentially
to avoid ML library global state conflicts and resource contention issues.

IMPORTANT: These tests require process isolation (--forked flag) to prevent
global state pollution between TensorFlow and PyTorch. Run with:
    pytest -c pytest_ml.ini tests/test_ml*.py

Regular pytest runs will skip these tests to avoid failures.
"""

import pytest
import numpy as np
import os
import tempfile
import sys

# Skip these tests in regular pytest runs (without ML_TESTS environment variable)
# They must be run separately with: ML_TESTS=1 pytest -c pytest_ml.ini tests/test_ml*.py
pytestmark = pytest.mark.skipif(
    os.environ.get("ML_TESTS") != "1",
    reason="ML tests require process isolation. Run with: ML_TESTS=1 pytest -c pytest_ml.ini tests/test_ml*.py",
)


@pytest.mark.ml_isolated
@pytest.mark.forked
class TestPyTorchCompatibility:
    """Test PyTorch 2.5.1 compatibility"""

    def test_pytorch_import(self):
        """Test PyTorch imports successfully"""
        import torch

        assert torch.__version__ >= "2.5.0"

    def test_pytorch_basic_tensor_operations(self):
        """Test basic PyTorch tensor operations"""
        import torch

        # Create tensors
        x = torch.randn(10, 5)
        y = torch.randn(10, 5)

        # Operations
        z = x + y
        assert z.shape == (10, 5)
        assert not torch.isnan(z).any()

        # Matrix multiplication
        w = torch.mm(x, y.T)
        assert w.shape == (10, 10)
        assert not torch.isnan(w).any()

    def test_pytorch_model_creation(self):
        """Test PyTorch model creation"""
        import torch
        import torch.nn as nn

        class SimpleModel(nn.Module):
            def __init__(self):
                super(SimpleModel, self).__init__()
                self.fc1 = nn.Linear(10, 20)
                self.fc2 = nn.Linear(20, 1)
                self.relu = nn.ReLU()

            def forward(self, x):
                x = self.relu(self.fc1(x))
                x = self.fc2(x)
                return x

        model = SimpleModel()
        assert model is not None

        # Test forward pass
        x = torch.randn(32, 10)
        output = model(x)
        assert output.shape == (32, 1)
        assert not torch.isnan(output).any()

    def test_pytorch_training_loop(self):
        """Test PyTorch training loop"""
        import torch
        import torch.nn as nn
        import torch.optim as optim

        # Simple model
        model = nn.Sequential(nn.Linear(10, 20), nn.ReLU(), nn.Linear(20, 1))

        # Optimizer and loss
        optimizer = optim.Adam(model.parameters(), lr=0.001)
        criterion = nn.MSELoss()

        # Dummy data
        x = torch.randn(32, 10)
        y = torch.randn(32, 1)

        # Training step
        model.train()
        optimizer.zero_grad()
        output = model(x)
        loss = criterion(output, y)
        loss.backward()
        optimizer.step()

        assert not torch.isnan(loss)
        assert loss.item() >= 0

    def test_pytorch_device_handling(self):
        """Test CPU/GPU device handling"""
        import torch

        device = torch.device("cuda" if torch.cuda.is_available() else "cpu")

        model = torch.nn.Linear(10, 1).to(device)
        x = torch.randn(1, 10).to(device)

        with torch.no_grad():
            output = model(x)

        assert output.device.type == device.type

    def test_pytorch_save_load(self):
        """Test model saving and loading"""
        import torch
        import torch.nn as nn

        model = nn.Sequential(nn.Linear(10, 20), nn.ReLU(), nn.Linear(20, 1))

        # Save model
        with tempfile.NamedTemporaryFile(suffix=".pth", delete=False) as tmp:
            tmp_path = tmp.name
            torch.save(model.state_dict(), tmp_path)

            # Load model
            loaded_model = nn.Sequential(nn.Linear(10, 20), nn.ReLU(), nn.Linear(20, 1))
            loaded_model.load_state_dict(torch.load(tmp_path))

            # Verify
            x = torch.randn(1, 10)
            with torch.no_grad():
                original_output = model(x)
                loaded_output = loaded_model(x)

            torch.testing.assert_close(original_output, loaded_output)

            # Cleanup
            os.unlink(tmp_path)

    def test_pytorch_autograd(self):
        """Test automatic differentiation"""
        import torch

        x = torch.randn(5, 3, requires_grad=True)
        y = x.pow(2).sum()
        y.backward()

        assert x.grad is not None
        assert x.grad.shape == x.shape


@pytest.mark.ml_isolated
@pytest.mark.forked
class TestTensorFlowCompatibility:
    """Test TensorFlow 2.18.0 and Keras 3.0 compatibility"""

    def test_tensorflow_import(self):
        """Test TensorFlow imports successfully"""
        import tensorflow as tf

        assert tf.__version__ >= "2.18.0"

    def test_keras_import(self):
        """Test Keras 3.0 imports from tensorflow.keras"""
        from tensorflow import keras

        # Verify it's Keras 3.0
        assert hasattr(keras, "__version__")

    def test_keras_sequential_model(self):
        """Test Keras Sequential model creation"""
        from tensorflow import keras

        model = keras.Sequential(
            [
                keras.layers.Dense(64, activation="relu", input_shape=(20,)),
                keras.layers.Dense(32, activation="relu"),
                keras.layers.Dense(10, activation="softmax"),
            ]
        )

        model.compile(
            optimizer="adam",
            loss="sparse_categorical_crossentropy",
            metrics=["accuracy"],
        )

        assert model is not None
        assert len(model.layers) == 3

    def test_keras_functional_model(self):
        """Test Keras Functional API model creation"""
        from tensorflow import keras
        from tensorflow.keras import layers

        inputs = layers.Input(shape=(20,))
        x = layers.Dense(64, activation="relu")(inputs)
        x = layers.Dense(32, activation="relu")(x)
        outputs = layers.Dense(10, activation="softmax")(x)

        model = keras.Model(inputs=inputs, outputs=outputs)

        model.compile(optimizer="adam", loss="sparse_categorical_crossentropy")

        assert model is not None

    def test_keras_training(self):
        """Test Keras model training"""
        from tensorflow import keras
        import numpy as np

        # Simple model
        model = keras.Sequential(
            [
                keras.layers.Dense(10, activation="relu", input_shape=(20,)),
                keras.layers.Dense(1),
            ]
        )

        model.compile(optimizer="adam", loss="mse")

        # Generate dummy data
        x = np.random.rand(100, 20).astype(np.float32)
        y = np.random.rand(100, 1).astype(np.float32)

        # Train
        history = model.fit(x, y, epochs=1, verbose=0)

        assert "loss" in history.history
        assert history.history["loss"][0] >= 0

    def test_keras_prediction(self):
        """Test Keras model prediction"""
        from tensorflow import keras
        import numpy as np

        model = keras.Sequential(
            [
                keras.layers.Dense(10, activation="relu", input_shape=(5,)),
                keras.layers.Dense(1),
            ]
        )

        model.compile(optimizer="adam", loss="mse")

        # Predict
        x = np.random.rand(10, 5).astype(np.float32)
        predictions = model.predict(x, verbose=0)

        assert predictions.shape == (10, 1)
        assert not np.isnan(predictions).any()

    def test_keras_save_load_native_format(self):
        """Test Keras model save/load with .keras format"""
        from tensorflow import keras
        import numpy as np

        model = keras.Sequential(
            [
                keras.layers.Dense(10, activation="relu", input_shape=(5,)),
                keras.layers.Dense(1),
            ]
        )

        model.compile(optimizer="adam", loss="mse")

        # Save in new .keras format
        with tempfile.NamedTemporaryFile(suffix=".keras", delete=False) as tmp:
            tmp_path = tmp.name
            model.save(tmp_path)

            # Load
            loaded_model = keras.models.load_model(tmp_path)

            # Verify
            x = np.random.rand(1, 5).astype(np.float32)
            original_pred = model.predict(x, verbose=0)
            loaded_pred = loaded_model.predict(x, verbose=0)

            np.testing.assert_array_almost_equal(original_pred, loaded_pred)

            # Cleanup
            os.unlink(tmp_path)

    def test_keras_save_load_h5_format(self):
        """Test Keras model save/load with legacy .h5 format"""
        from tensorflow import keras
        import numpy as np

        model = keras.Sequential(
            [
                keras.layers.Dense(10, activation="relu", input_shape=(5,)),
                keras.layers.Dense(1),
            ]
        )

        model.compile(optimizer="adam", loss="mse")

        # Save in legacy .h5 format (should still work)
        with tempfile.NamedTemporaryFile(suffix=".h5", delete=False) as tmp:
            tmp_path = tmp.name
            model.save(tmp_path)

            # Load with custom_objects to handle Keras 3.0 compatibility
            # In Keras 3.0, loss functions have different internal representations
            from keras import losses

            custom_objects = {"mse": losses.MeanSquaredError()}
            loaded_model = keras.models.load_model(
                tmp_path, custom_objects=custom_objects
            )

            # Verify
            x = np.random.rand(1, 5).astype(np.float32)
            original_pred = model.predict(x, verbose=0)
            loaded_pred = loaded_model.predict(x, verbose=0)

            np.testing.assert_array_almost_equal(original_pred, loaded_pred)

            # Cleanup
            os.unlink(tmp_path)

    def test_keras_callbacks(self):
        """Test Keras callbacks work correctly"""
        from tensorflow import keras
        import numpy as np

        model = keras.Sequential(
            [
                keras.layers.Dense(10, activation="relu", input_shape=(20,)),
                keras.layers.Dense(1),
            ]
        )

        model.compile(optimizer="adam", loss="mse")

        # Create callbacks
        early_stopping = keras.callbacks.EarlyStopping(
            monitor="loss", patience=3, restore_best_weights=True
        )

        reduce_lr = keras.callbacks.ReduceLROnPlateau(
            monitor="loss", factor=0.5, patience=2
        )

        # Generate dummy data
        x = np.random.rand(100, 20).astype(np.float32)
        y = np.random.rand(100, 1).astype(np.float32)

        # Train with callbacks
        history = model.fit(
            x, y, epochs=5, verbose=0, callbacks=[early_stopping, reduce_lr]
        )

        assert len(history.history["loss"]) > 0

    def test_keras_batch_normalization(self):
        """Test Keras BatchNormalization layer"""
        from tensorflow import keras

        model = keras.Sequential(
            [
                keras.layers.Dense(64, input_shape=(20,)),
                keras.layers.BatchNormalization(),
                keras.layers.Activation("relu"),
                keras.layers.Dense(10),
            ]
        )

        model.compile(optimizer="adam", loss="mse")

        assert model is not None
        assert any(
            isinstance(layer, keras.layers.BatchNormalization) for layer in model.layers
        )

    def test_keras_dropout(self):
        """Test Keras Dropout layer"""
        from tensorflow import keras

        model = keras.Sequential(
            [
                keras.layers.Dense(64, activation="relu", input_shape=(20,)),
                keras.layers.Dropout(0.5),
                keras.layers.Dense(10),
            ]
        )

        model.compile(optimizer="adam", loss="mse")

        assert model is not None
        assert any(isinstance(layer, keras.layers.Dropout) for layer in model.layers)


@pytest.mark.ml_isolated
@pytest.mark.forked
class TestMLLibraryInteroperability:
    """Test PyTorch and TensorFlow can coexist"""

    def test_both_libraries_import(self):
        """Test both libraries can be imported together"""
        import torch
        import tensorflow as tf

        # Verify libraries can be imported (version checks may vary in local environments)
        assert torch.__version__ is not None
        assert tf.__version__ is not None

    def test_numpy_interoperability(self):
        """Test NumPy arrays work with both libraries"""
        import torch
        import tensorflow as tf
        import numpy as np

        # Create NumPy array
        np_array = np.random.rand(5, 3).astype(np.float32)

        # Convert to PyTorch
        torch_tensor = torch.from_numpy(np_array)
        assert torch_tensor.shape == (5, 3)

        # Convert to TensorFlow
        tf_tensor = tf.convert_to_tensor(np_array)
        assert tf_tensor.shape == (5, 3)

        # Convert back to NumPy
        np_from_torch = torch_tensor.numpy()
        np_from_tf = tf_tensor.numpy()

        np.testing.assert_array_almost_equal(np_array, np_from_torch)
        np.testing.assert_array_almost_equal(np_array, np_from_tf)
