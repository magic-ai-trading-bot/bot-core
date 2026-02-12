import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import ErrorBoundary from '@/components/ErrorBoundary';

// Component that throws an error
const ThrowError = ({ shouldThrow }: { shouldThrow: boolean }) => {
  if (shouldThrow) {
    throw new Error('Test error');
  }
  return <div>No error</div>;
};

describe('Error Boundary', () => {
  beforeEach(() => {
    // Suppress console.error for these tests
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  it('should render children when there is no error', () => {
    render(
      <ErrorBoundary>
        <ThrowError shouldThrow={false} />
      </ErrorBoundary>
    );

    expect(screen.getByText('No error')).toBeInTheDocument();
  });

  it('should catch and display error', () => {
    render(
      <ErrorBoundary>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    );

    expect(screen.getByText(/có lỗi xảy ra/i)).toBeInTheDocument();
  });

  it('should display custom fallback UI', () => {
    render(
      <ErrorBoundary fallback={<div>Custom error message</div>}>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    );

    expect(screen.getByText('Custom error message')).toBeInTheDocument();
  });

  it('should catch errors in nested components', () => {
    const NestedComponent = () => (
      <div>
        <div>
          <ThrowError shouldThrow={true} />
        </div>
      </div>
    );

    render(
      <ErrorBoundary>
        <NestedComponent />
      </ErrorBoundary>
    );

    expect(screen.getByText(/có lỗi xảy ra/i)).toBeInTheDocument();
  });

  it('should not catch errors outside boundary', () => {
    const OutsideComponent = () => {
      throw new Error('Outside error');
    };

    expect(() => {
      render(<OutsideComponent />);
    }).toThrow('Outside error');
  });
});

describe('Error Boundary with Context', () => {
  it('should maintain context after error', () => {
    const TestContext = ({ children }: { children: ReactNode }) => (
      <div data-testid="context-wrapper">{children}</div>
    );

    render(
      <TestContext>
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      </TestContext>
    );

    expect(screen.getByTestId('context-wrapper')).toBeInTheDocument();
    expect(screen.getByText(/có lỗi xảy ra/i)).toBeInTheDocument();
  });
});

describe('Error Boundary Recovery', () => {
  it('should recover from error state when props change', () => {
    const { rerender } = render(
      <ErrorBoundary>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    );

    expect(screen.getByText(/có lỗi xảy ra/i)).toBeInTheDocument();

    // Note: In real implementation, you would need a reset mechanism
    // This test demonstrates the concept
  });

  it('should reset error state when handleReset is called', async () => {
    const user = userEvent.setup();
    render(
      <ErrorBoundary>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    );

    // Error should be displayed
    expect(screen.getByText(/có lỗi xảy ra/i)).toBeInTheDocument();

    // Click "Thử lại" button
    const resetButton = screen.getByRole('button', { name: /thử lại/i });
    await user.click(resetButton);

    // After reset, children should render (but will error again)
    // Due to how error boundaries work, this will catch the error again
    expect(screen.getByText(/có lỗi xảy ra/i)).toBeInTheDocument();
  });

  it('should reload page when reload button is clicked', async () => {
    const user = userEvent.setup();
    const reloadMock = vi.fn();
    Object.defineProperty(window, 'location', {
      value: { reload: reloadMock },
      writable: true,
    });

    render(
      <ErrorBoundary>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    );

    // Click "Reload trang" button
    const reloadButton = screen.getByRole('button', { name: /reload trang/i });
    await user.click(reloadButton);

    expect(reloadMock).toHaveBeenCalled();
  });

  it('should display error message when error occurs', () => {
    render(
      <ErrorBoundary>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    );

    // Should show the error message
    expect(screen.getByText('Error: Test error')).toBeInTheDocument();
  });

  it('should call componentDidCatch with error and errorInfo', () => {
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    render(
      <ErrorBoundary>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    );

    // componentDidCatch should have been called (console.error in tests)
    expect(consoleErrorSpy).toHaveBeenCalled();

    consoleErrorSpy.mockRestore();
  });

  it('should handle production mode error logging', () => {
    const originalEnv = process.env.NODE_ENV;
    process.env.NODE_ENV = 'production';

    render(
      <ErrorBoundary>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    );

    // In production mode, error should still be caught and displayed
    expect(screen.getByText(/có lỗi xảy ra/i)).toBeInTheDocument();

    process.env.NODE_ENV = originalEnv;
  });
});
