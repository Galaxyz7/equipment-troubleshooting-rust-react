import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import LandingPage from './LandingPage';

const mockNavigate = vi.fn();

// Mock react-router-dom
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

/**
 * LandingPage Component Tests
 * Tests the main landing page rendering and navigation
 */

describe('LandingPage', () => {
  beforeEach(() => {
    mockNavigate.mockClear();
  });

  it('should render the main heading', () => {
    render(
      <MemoryRouter>
        <LandingPage />
      </MemoryRouter>
    );

    expect(screen.getByText(/Equipment Troubleshooting/i)).toBeInTheDocument();
  });

  it('should render the description text', () => {
    render(
      <MemoryRouter>
        <LandingPage />
      </MemoryRouter>
    );

    expect(
      screen.getByText(/This tool will guide you through a series of questions/i)
    ).toBeInTheDocument();
  });

  it('should render the Start Troubleshooting button', () => {
    render(
      <MemoryRouter>
        <LandingPage />
      </MemoryRouter>
    );

    const button = screen.getByRole('button', { name: /Start Troubleshooting/i });
    expect(button).toBeInTheDocument();
  });

  it('should render the Admin Login link', () => {
    render(
      <MemoryRouter>
        <LandingPage />
      </MemoryRouter>
    );

    const link = screen.getByRole('link', { name: /Admin Login/i });
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute('href', '/admin/login');
  });

  it('should navigate to troubleshoot page when button is clicked', async () => {
    const user = userEvent.setup();

    render(
      <MemoryRouter>
        <LandingPage />
      </MemoryRouter>
    );

    const button = screen.getByRole('button', { name: /Start Troubleshooting/i });
    await user.click(button);

    expect(mockNavigate).toHaveBeenCalledWith('/troubleshoot');
  });

  it('should have proper styling classes', () => {
    render(
      <MemoryRouter>
        <LandingPage />
      </MemoryRouter>
    );

    const container = screen.getByText(/Equipment Troubleshooting/i).closest('div');
    expect(container).toHaveClass('bg-white', 'rounded-xl');
  });
});
