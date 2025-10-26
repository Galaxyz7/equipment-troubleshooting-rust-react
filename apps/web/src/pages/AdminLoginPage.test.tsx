import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import AdminLoginPage from './AdminLoginPage';
import type { UserRole } from '../types';

const mockNavigate = vi.fn();

// Mock react-router-dom
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

// Mock the API
vi.mock('../lib/api', () => ({
  authAPI: {
    login: vi.fn(),
  },
}));

import { authAPI } from '../lib/api';

/**
 * AdminLoginPage Component Tests
 * Tests form rendering, validation, and submission
 */

describe('AdminLoginPage', () => {
  beforeEach(() => {
    mockNavigate.mockClear();
    vi.mocked(authAPI.login).mockClear();
    localStorage.clear();
  });

  it('should render the login form', () => {
    render(
      <MemoryRouter>
        <AdminLoginPage />
      </MemoryRouter>
    );

    expect(screen.getByRole('heading', { name: /Admin Login/i })).toBeInTheDocument();
    expect(screen.getByLabelText(/Email/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Password/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Login/i })).toBeInTheDocument();
  });

  it('should have email and password inputs', () => {
    render(
      <MemoryRouter>
        <AdminLoginPage />
      </MemoryRouter>
    );

    const emailInput = screen.getByLabelText(/Email/i);
    const passwordInput = screen.getByLabelText(/Password/i);

    expect(emailInput).toHaveAttribute('type', 'email');
    expect(passwordInput).toHaveAttribute('type', 'password');
    expect(emailInput).toHaveAttribute('required');
    expect(passwordInput).toHaveAttribute('required');
  });

  it('should update email input value when typed', async () => {
    const user = userEvent.setup();

    render(
      <MemoryRouter>
        <AdminLoginPage />
      </MemoryRouter>
    );

    const emailInput = screen.getByLabelText(/Email/i) as HTMLInputElement;
    await user.type(emailInput, 'admin@example.com');

    expect(emailInput.value).toBe('admin@example.com');
  });

  it('should update password input value when typed', async () => {
    const user = userEvent.setup();

    render(
      <MemoryRouter>
        <AdminLoginPage />
      </MemoryRouter>
    );

    const passwordInput = screen.getByLabelText(/Password/i) as HTMLInputElement;
    await user.type(passwordInput, 'password123');

    expect(passwordInput.value).toBe('password123');
  });

  it('should call login API when form is submitted', async () => {
    const user = userEvent.setup();
    const mockResponse = {
      token: 'test-token',
      user: { id: '1', email: 'admin@example.com', role: 'Admin' as UserRole },
    };

    vi.mocked(authAPI.login).mockResolvedValue(mockResponse);

    render(
      <MemoryRouter>
        <AdminLoginPage />
      </MemoryRouter>
    );

    const emailInput = screen.getByLabelText(/Email/i);
    const passwordInput = screen.getByLabelText(/Password/i);
    const submitButton = screen.getByRole('button', { name: /Login/i });

    await user.type(emailInput, 'admin@example.com');
    await user.type(passwordInput, 'password123');
    await user.click(submitButton);

    await waitFor(() => {
      expect(authAPI.login).toHaveBeenCalledWith({
        email: 'admin@example.com',
        password: 'password123',
      });
    });
  });

  it('should store token and navigate on successful login', async () => {
    const user = userEvent.setup();
    const mockResponse = {
      token: 'test-jwt-token',
      user: { id: '1', email: 'admin@example.com', role: 'Admin' as UserRole },
    };

    vi.mocked(authAPI.login).mockResolvedValue(mockResponse);

    render(
      <MemoryRouter>
        <AdminLoginPage />
      </MemoryRouter>
    );

    const emailInput = screen.getByLabelText(/Email/i);
    const passwordInput = screen.getByLabelText(/Password/i);
    const submitButton = screen.getByRole('button', { name: /Login/i });

    await user.type(emailInput, 'admin@example.com');
    await user.type(passwordInput, 'password123');
    await user.click(submitButton);

    await waitFor(() => {
      expect(localStorage.getItem('token')).toBe('test-jwt-token');
      expect(mockNavigate).toHaveBeenCalledWith('/admin');
    });
  });

  it('should display error message on failed login', async () => {
    const user = userEvent.setup();
    const mockError = {
      response: {
        data: {
          error: {
            data: {
              message: 'Invalid email or password',
            },
          },
        },
      },
    };

    vi.mocked(authAPI.login).mockRejectedValue(mockError);

    render(
      <MemoryRouter>
        <AdminLoginPage />
      </MemoryRouter>
    );

    const emailInput = screen.getByLabelText(/Email/i);
    const passwordInput = screen.getByLabelText(/Password/i);
    const submitButton = screen.getByRole('button', { name: /Login/i });

    await user.type(emailInput, 'wrong@example.com');
    await user.type(passwordInput, 'wrongpassword');
    await user.click(submitButton);

    await waitFor(() => {
      expect(screen.getByText(/Invalid email or password/i)).toBeInTheDocument();
    });
  });

  it('should disable submit button while loading', async () => {
    const user = userEvent.setup();

    // Create a promise that never resolves to keep loading state
    const loginPromise = new Promise(() => {
      // Never resolves to keep loading state
    });

    vi.mocked(authAPI.login).mockReturnValue(loginPromise as any);

    render(
      <MemoryRouter>
        <AdminLoginPage />
      </MemoryRouter>
    );

    const emailInput = screen.getByLabelText(/Email/i);
    const passwordInput = screen.getByLabelText(/Password/i);
    const submitButton = screen.getByRole('button', { name: /Login/i });

    await user.type(emailInput, 'admin@example.com');
    await user.type(passwordInput, 'password123');
    await user.click(submitButton);

    await waitFor(() => {
      expect(submitButton).toBeDisabled();
    });
  });

  it('should have home link', () => {
    render(
      <MemoryRouter>
        <AdminLoginPage />
      </MemoryRouter>
    );

    const homeLink = screen.getByRole('link', { name: /Back to Home/i });
    expect(homeLink).toBeInTheDocument();
    expect(homeLink).toHaveAttribute('href', '/');
  });

  it('should clear error when form is resubmitted', async () => {
    const user = userEvent.setup();

    // First submission fails
    vi.mocked(authAPI.login).mockRejectedValueOnce(new Error('Network error'));

    render(
      <MemoryRouter>
        <AdminLoginPage />
      </MemoryRouter>
    );

    const emailInput = screen.getByLabelText(/Email/i);
    const passwordInput = screen.getByLabelText(/Password/i);
    const submitButton = screen.getByRole('button', { name: /Login/i });

    await user.type(emailInput, 'admin@example.com');
    await user.type(passwordInput, 'pass');
    await user.click(submitButton);

    // Wait for error to appear
    await waitFor(() => {
      expect(screen.getByText(/Invalid email or password/i)).toBeInTheDocument();
    });

    // Second submission should clear error before making request
    const mockSuccess = {
      token: 'new-token',
      user: { id: '1', email: 'admin@example.com', role: 'Admin' as UserRole },
    };
    vi.mocked(authAPI.login).mockResolvedValueOnce(mockSuccess);

    await user.click(submitButton);

    // Error message should not be visible during loading
    // (it gets cleared in handleSubmit before the API call)
    await waitFor(() => {
      expect(screen.queryByText(/Invalid email or password/i)).not.toBeInTheDocument();
    });
  });
});
