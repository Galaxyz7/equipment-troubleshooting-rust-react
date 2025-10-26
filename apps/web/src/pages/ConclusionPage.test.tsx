import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import ConclusionPage from './ConclusionPage';
import type { HistoryStep } from '../types/troubleshoot';

const mockNavigate = vi.fn();

// Mock react-router-dom
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
    useLocation: vi.fn(),
  };
});

import { useLocation } from 'react-router-dom';

/**
 * ConclusionPage Component Tests
 * Tests the conclusion/diagnosis results page
 */

const mockHistory: HistoryStep[] = [
  {
    question: { text: 'Is the device powered on?' },
    answer: { label: 'No' },
  } as HistoryStep,
  {
    question: { text: 'Is the power cable connected?' },
    answer: { label: 'Yes' },
  } as HistoryStep,
];

describe('ConclusionPage', () => {
  beforeEach(() => {
    mockNavigate.mockClear();
    vi.mocked(useLocation).mockClear();
  });

  it('should display conclusion text', () => {
    vi.mocked(useLocation).mockReturnValue({
      state: {
        conclusion: 'Replace the power supply',
        history: [],
      },
      pathname: '/conclusion',
      search: '',
      hash: '',
      key: 'default',
    });

    render(
      <MemoryRouter>
        <ConclusionPage />
      </MemoryRouter>
    );

    expect(screen.getByText('Replace the power supply')).toBeInTheDocument();
  });

  it('should display success heading', () => {
    vi.mocked(useLocation).mockReturnValue({
      state: {
        conclusion: 'Replace the power supply',
        history: [],
      },
      pathname: '/conclusion',
      search: '',
      hash: '',
      key: 'default',
    });

    render(
      <MemoryRouter>
        <ConclusionPage />
      </MemoryRouter>
    );

    expect(screen.getByText('Diagnosis Complete')).toBeInTheDocument();
  });

  it('should display action buttons', () => {
    vi.mocked(useLocation).mockReturnValue({
      state: {
        conclusion: 'Test conclusion',
        history: [],
      },
      pathname: '/conclusion',
      search: '',
      hash: '',
      key: 'default',
    });

    render(
      <MemoryRouter>
        <ConclusionPage />
      </MemoryRouter>
    );

    expect(screen.getByRole('button', { name: /Start New Diagnosis/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Back to Home/i })).toBeInTheDocument();
  });

  it('should navigate to troubleshoot page when Start New Diagnosis is clicked', async () => {
    const user = userEvent.setup();

    vi.mocked(useLocation).mockReturnValue({
      state: {
        conclusion: 'Test conclusion',
        history: [],
      },
      pathname: '/conclusion',
      search: '',
      hash: '',
      key: 'default',
    });

    render(
      <MemoryRouter>
        <ConclusionPage />
      </MemoryRouter>
    );

    const button = screen.getByRole('button', { name: /Start New Diagnosis/i });
    await user.click(button);

    expect(mockNavigate).toHaveBeenCalledWith('/troubleshoot');
  });

  it('should navigate to home when Back to Home is clicked', async () => {
    const user = userEvent.setup();

    vi.mocked(useLocation).mockReturnValue({
      state: {
        conclusion: 'Test conclusion',
        history: [],
      },
      pathname: '/conclusion',
      search: '',
      hash: '',
      key: 'default',
    });

    render(
      <MemoryRouter>
        <ConclusionPage />
      </MemoryRouter>
    );

    const button = screen.getByRole('button', { name: /Back to Home/i });
    await user.click(button);

    expect(mockNavigate).toHaveBeenCalledWith('/');
  });

  it('should display diagnostic path when history is provided', () => {
    vi.mocked(useLocation).mockReturnValue({
      state: {
        conclusion: 'Test conclusion',
        history: mockHistory,
      },
      pathname: '/conclusion',
      search: '',
      hash: '',
      key: 'default',
    });

    render(
      <MemoryRouter>
        <ConclusionPage />
      </MemoryRouter>
    );

    expect(screen.getByText('📋 Diagnostic Path')).toBeInTheDocument();
    expect(screen.getByText('Is the device powered on?')).toBeInTheDocument();
    expect(screen.getByText('→ No')).toBeInTheDocument();
    expect(screen.getByText('Is the power cable connected?')).toBeInTheDocument();
    expect(screen.getByText('→ Yes')).toBeInTheDocument();
  });

  it('should not display diagnostic path when history is empty', () => {
    vi.mocked(useLocation).mockReturnValue({
      state: {
        conclusion: 'Test conclusion',
        history: [],
      },
      pathname: '/conclusion',
      search: '',
      hash: '',
      key: 'default',
    });

    render(
      <MemoryRouter>
        <ConclusionPage />
      </MemoryRouter>
    );

    expect(screen.queryByText('📋 Diagnostic Path')).not.toBeInTheDocument();
  });

  it('should display Print Report button when history exists', () => {
    vi.mocked(useLocation).mockReturnValue({
      state: {
        conclusion: 'Test conclusion',
        history: mockHistory,
      },
      pathname: '/conclusion',
      search: '',
      hash: '',
      key: 'default',
    });

    render(
      <MemoryRouter>
        <ConclusionPage />
      </MemoryRouter>
    );

    expect(screen.getByRole('button', { name: /Print Report/i })).toBeInTheDocument();
  });

  it('should call window.print when Print Report is clicked', async () => {
    const user = userEvent.setup();
    const mockPrint = vi.fn();
    vi.stubGlobal('print', mockPrint);

    vi.mocked(useLocation).mockReturnValue({
      state: {
        conclusion: 'Test conclusion',
        history: mockHistory,
      },
      pathname: '/conclusion',
      search: '',
      hash: '',
      key: 'default',
    });

    render(
      <MemoryRouter>
        <ConclusionPage />
      </MemoryRouter>
    );

    const printButton = screen.getByRole('button', { name: /Print Report/i });
    await user.click(printButton);

    expect(mockPrint).toHaveBeenCalled();

    vi.unstubAllGlobals();
  });

  it('should redirect to home when no conclusion is provided', () => {
    vi.mocked(useLocation).mockReturnValue({
      state: null,
      pathname: '/conclusion',
      search: '',
      hash: '',
      key: 'default',
    });

    render(
      <MemoryRouter>
        <ConclusionPage />
      </MemoryRouter>
    );

    expect(mockNavigate).toHaveBeenCalledWith('/');
  });

  it('should display success icon', () => {
    vi.mocked(useLocation).mockReturnValue({
      state: {
        conclusion: 'Test conclusion',
        history: [],
      },
      pathname: '/conclusion',
      search: '',
      hash: '',
      key: 'default',
    });

    render(
      <MemoryRouter>
        <ConclusionPage />
      </MemoryRouter>
    );

    expect(screen.getByText('✅')).toBeInTheDocument();
  });

  it('should preserve whitespace in conclusion text', () => {
    const multilineConclusion = 'Line 1\nLine 2\nLine 3';

    vi.mocked(useLocation).mockReturnValue({
      state: {
        conclusion: multilineConclusion,
        history: [],
      },
      pathname: '/conclusion',
      search: '',
      hash: '',
      key: 'default',
    });

    render(
      <MemoryRouter>
        <ConclusionPage />
      </MemoryRouter>
    );

    const conclusionElement = screen.getByText(/Line 1/);
    expect(conclusionElement).toHaveClass('whitespace-pre-line');
  });
});
