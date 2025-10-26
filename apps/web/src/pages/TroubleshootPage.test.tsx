import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import TroubleshootPage from './TroubleshootPage';
import type { Node, NavigationOption } from '../types';

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
  troubleshootAPI: {
    startSession: vi.fn(),
    submitAnswer: vi.fn(),
  },
}));

import { troubleshootAPI } from '../lib/api';

/**
 * TroubleshootPage Component Tests
 * Tests the troubleshooting flow, navigation, and state management
 */

const mockQuestionNode: Node = {
  id: 'node-1',
  category: 'hardware',
  node_type: 'Question',
  text: 'Is the device powered on?',
  semantic_id: 'hw_power',
  display_category: 'Hardware',
  position_x: 100,
  position_y: 200,
  is_active: true,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

const mockOptions: NavigationOption[] = [
  {
    connection_id: 'conn-1',
    label: 'Yes',
    target_category: 'hardware',
    display_category: 'Hardware',
  },
  {
    connection_id: 'conn-2',
    label: 'No',
    target_category: 'hardware',
    display_category: 'Hardware',
  },
];

const mockConclusionNode: Node = {
  id: 'node-conclusion',
  category: 'hardware',
  node_type: 'Conclusion',
  text: 'Replace the power supply',
  semantic_id: null,
  display_category: 'Hardware',
  position_x: null,
  position_y: null,
  is_active: true,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

describe('TroubleshootPage', () => {
  beforeEach(() => {
    mockNavigate.mockClear();
    vi.mocked(troubleshootAPI.startSession).mockClear();
    vi.mocked(troubleshootAPI.submitAnswer).mockClear();
  });

  it('should start session on mount', async () => {
    const mockResponse = {
      session_id: 'session-123',
      node: mockQuestionNode,
      options: mockOptions,
    };

    vi.mocked(troubleshootAPI.startSession).mockResolvedValue(mockResponse);

    render(
      <MemoryRouter initialEntries={['/troubleshoot']}>
        <Routes>
          <Route path="/troubleshoot" element={<TroubleshootPage />} />
        </Routes>
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(troubleshootAPI.startSession).toHaveBeenCalledWith({
        tech_identifier: null,
        client_site: null,
        category: null,
      });
    });
  });

  it('should display question text when session loads', async () => {
    const mockResponse = {
      session_id: 'session-123',
      node: mockQuestionNode,
      options: mockOptions,
    };

    vi.mocked(troubleshootAPI.startSession).mockResolvedValue(mockResponse);

    render(
      <MemoryRouter initialEntries={['/troubleshoot']}>
        <Routes>
          <Route path="/troubleshoot" element={<TroubleshootPage />} />
        </Routes>
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(screen.getByText('Is the device powered on?')).toBeInTheDocument();
    });
  });

  it('should display navigation options', async () => {
    const mockResponse = {
      session_id: 'session-123',
      node: mockQuestionNode,
      options: mockOptions,
    };

    vi.mocked(troubleshootAPI.startSession).mockResolvedValue(mockResponse);

    render(
      <MemoryRouter initialEntries={['/troubleshoot']}>
        <Routes>
          <Route path="/troubleshoot" element={<TroubleshootPage />} />
        </Routes>
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(screen.getByText('Yes')).toBeInTheDocument();
      expect(screen.getByText('No')).toBeInTheDocument();
    });
  });

//   it('should select option when clicked', async () => {
//     const user = userEvent.setup();
//     const mockResponse = {
//       session_id: 'session-123',
//       node: mockQuestionNode,
//       options: mockOptions,
//     };
// 
//     vi.mocked(troubleshootAPI.startSession).mockResolvedValue(mockResponse);
// 
//     render(
//       <MemoryRouter initialEntries={['/troubleshoot']}>
//         <Routes>
//           <Route path="/troubleshoot" element={<TroubleshootPage />} />
//         </Routes>
//       </MemoryRouter>
//     );
// 
//     await waitFor(() => {
//       expect(screen.getByText('Yes')).toBeInTheDocument();
//     });
// 
//     const yesOption = screen.getByText('Yes').closest('div[class*="cursor-pointer"]');
//     await user.click(yesOption!);
// 
//     // Option should be selected (visual feedback)
//     expect(yesOption).toHaveClass('border-[#667eea]');
//   });

//   it('should submit answer when Next button is clicked', async () => {
//     const user = userEvent.setup();
//     const startResponse = {
//       session_id: 'session-123',
//       node: mockQuestionNode,
//       options: mockOptions,
//     };
// 
//     const nextNode: Node = {
//       ...mockQuestionNode,
//       id: 'node-2',
//       text: 'Is the power cable connected?',
//     };
// 
//     const submitResponse = {
//       node: nextNode,
//       options: mockOptions,
//     };
// 
//     vi.mocked(troubleshootAPI.startSession).mockResolvedValue(startResponse);
//     vi.mocked(troubleshootAPI.submitAnswer).mockResolvedValue(submitResponse);
// 
//     render(
//       <MemoryRouter initialEntries={['/troubleshoot']}>
//         <Routes>
//           <Route path="/troubleshoot" element={<TroubleshootPage />} />
//         </Routes>
//       </MemoryRouter>
//     );
// 
//     await waitFor(() => {
//       expect(screen.getByText('Yes')).toBeInTheDocument();
//     });
// 
//     const yesOption = screen.getByText('Yes').closest('div[class*="cursor-pointer"]');
//     await user.click(yesOption!);
// 
//     const nextButton = screen.getByRole('button', { name: /Next/i });
//     await user.click(nextButton);
// 
//     await waitFor(() => {
//       expect(troubleshootAPI.submitAnswer).toHaveBeenCalledWith('session-123', {
//         connection_id: 'conn-1',
//       });
//     });
// 
//     await waitFor(() => {
//       expect(screen.getByText('Is the power cable connected?')).toBeInTheDocument();
//     });
//   });

  it('should display loading state while starting session', async () => {
    const sessionPromise = new Promise(() => {
      // Never resolves to keep loading state
    });

    vi.mocked(troubleshootAPI.startSession).mockReturnValue(sessionPromise as any);

    render(
      <MemoryRouter initialEntries={['/troubleshoot']}>
        <Routes>
          <Route path="/troubleshoot" element={<TroubleshootPage />} />
        </Routes>
      </MemoryRouter>
    );

    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('should display error message when session fails', async () => {
    vi.mocked(troubleshootAPI.startSession).mockRejectedValue(
      new Error('Network error')
    );

    render(
      <MemoryRouter initialEntries={['/troubleshoot']}>
        <Routes>
          <Route path="/troubleshoot" element={<TroubleshootPage />} />
        </Routes>
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(screen.getByText(/Failed to start session/i)).toBeInTheDocument();
    });
  });

//   it('should display error message when answer submission fails', async () => {
//     const user = userEvent.setup();
//     const startResponse = {
//       session_id: 'session-123',
//       node: mockQuestionNode,
//       options: mockOptions,
//     };
// 
//     vi.mocked(troubleshootAPI.startSession).mockResolvedValue(startResponse);
//     vi.mocked(troubleshootAPI.submitAnswer).mockRejectedValue(
//       new Error('Network error')
//     );
// 
//     render(
//       <MemoryRouter initialEntries={['/troubleshoot']}>
//         <Routes>
//           <Route path="/troubleshoot" element={<TroubleshootPage />} />
//         </Routes>
//       </MemoryRouter>
//     );
// 
//     await waitFor(() => {
//       expect(screen.getByText('Yes')).toBeInTheDocument();
//     });
// 
//     const yesOption = screen.getByText('Yes').closest('div[class*="cursor-pointer"]');
//     await user.click(yesOption!);
// 
//     const nextButton = screen.getByRole('button', { name: /Next/i });
//     await user.click(nextButton);
// 
//     await waitFor(() => {
//       expect(screen.getByText(/Failed to submit answer/i)).toBeInTheDocument();
//     });
//   });

//   it('should navigate back when Back button is clicked', async () => {
//     const user = userEvent.setup();
//     const startResponse = {
//       session_id: 'session-123',
//       node: mockQuestionNode,
//       options: mockOptions,
//     };
// 
//     const nextNode: Node = {
//       ...mockQuestionNode,
//       id: 'node-2',
//       text: 'Is the power cable connected?',
//     };
// 
//     const submitResponse = {
//       node: nextNode,
//       options: mockOptions,
//     };
// 
//     vi.mocked(troubleshootAPI.startSession).mockResolvedValue(startResponse);
//     vi.mocked(troubleshootAPI.submitAnswer).mockResolvedValue(submitResponse);
// 
//     render(
//       <MemoryRouter initialEntries={['/troubleshoot']}>
//         <Routes>
//           <Route path="/troubleshoot" element={<TroubleshootPage />} />
//         </Routes>
//       </MemoryRouter>
//     );
// 
//     // Wait for initial load
//     await waitFor(() => {
//       expect(screen.getByText('Is the device powered on?')).toBeInTheDocument();
//     });
// 
//     // Select option and submit
//     const yesOption = screen.getByText('Yes').closest('div[class*="cursor-pointer"]');
//     await user.click(yesOption!);
// 
//     const nextButton = screen.getByRole('button', { name: /Next/i });
//     await user.click(nextButton);
// 
//     // Wait for next question
//     await waitFor(() => {
//       expect(screen.getByText('Is the power cable connected?')).toBeInTheDocument();
//     });
// 
//     // Click back button
//     const backButton = screen.getByRole('button', { name: /Back/i });
//     await user.click(backButton);
// 
//     // Should return to previous question
//     await waitFor(() => {
//       expect(screen.getByText('Is the device powered on?')).toBeInTheDocument();
//     });
//   });

//   it('should display conclusion when reaching end of flow', async () => {
//     const user = userEvent.setup();
//     const startResponse = {
//       session_id: 'session-123',
//       node: mockQuestionNode,
//       options: mockOptions,
//     };
// 
//     const conclusionResponse = {
//       node: mockConclusionNode,
//       options: [],
//     };
// 
//     vi.mocked(troubleshootAPI.startSession).mockResolvedValue(startResponse);
//     vi.mocked(troubleshootAPI.submitAnswer).mockResolvedValue(conclusionResponse);
// 
//     render(
//       <MemoryRouter initialEntries={['/troubleshoot']}>
//         <Routes>
//           <Route path="/troubleshoot" element={<TroubleshootPage />} />
//         </Routes>
//       </MemoryRouter>
//     );
// 
//     await waitFor(() => {
//       expect(screen.getByText('Is the device powered on?')).toBeInTheDocument();
//     });
// 
//     const noOption = screen.getByText('No').closest('div[class*="cursor-pointer"]');
//     await user.click(noOption!);
// 
//     const nextButton = screen.getByRole('button', { name: /Next/i });
//     await user.click(nextButton);
// 
//     await waitFor(() => {
//       expect(screen.getByText('Replace the power supply')).toBeInTheDocument();
//     });
// 
//     // Should show "Start Over" button for conclusions
//     expect(screen.getByRole('button', { name: /Start Over/i })).toBeInTheDocument();
//   });

  it('should restart session when Start Over is clicked', async () => {
    const user = userEvent.setup();
    const startResponse = {
      session_id: 'session-123',
      node: mockConclusionNode,
      options: [],
    };

    const restartResponse = {
      session_id: 'session-456',
      node: mockQuestionNode,
      options: mockOptions,
    };

    vi.mocked(troubleshootAPI.startSession)
      .mockResolvedValueOnce(startResponse)
      .mockResolvedValueOnce(restartResponse);

    render(
      <MemoryRouter initialEntries={['/troubleshoot']}>
        <Routes>
          <Route path="/troubleshoot" element={<TroubleshootPage />} />
        </Routes>
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(screen.getByText('Replace the power supply')).toBeInTheDocument();
    });

    const startOverButton = screen.getByRole('button', { name: /Start Over/i });
    await user.click(startOverButton);

    await waitFor(() => {
      expect(troubleshootAPI.startSession).toHaveBeenCalledTimes(2);
    });

    await waitFor(() => {
      expect(screen.getByText('Is the device powered on?')).toBeInTheDocument();
    });
  });

  it('should disable Next button when no option is selected', async () => {
    const mockResponse = {
      session_id: 'session-123',
      node: mockQuestionNode,
      options: mockOptions,
    };

    vi.mocked(troubleshootAPI.startSession).mockResolvedValue(mockResponse);

    render(
      <MemoryRouter initialEntries={['/troubleshoot']}>
        <Routes>
          <Route path="/troubleshoot" element={<TroubleshootPage />} />
        </Routes>
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(screen.getByText('Is the device powered on?')).toBeInTheDocument();
    });

    const nextButton = screen.getByRole('button', { name: /Next/i });
    expect(nextButton).toBeDisabled();
  });

  it('should start session with category from URL params', async () => {
    const mockResponse = {
      session_id: 'session-123',
      node: mockQuestionNode,
      options: mockOptions,
    };

    vi.mocked(troubleshootAPI.startSession).mockResolvedValue(mockResponse);

    render(
      <MemoryRouter initialEntries={['/troubleshoot/hardware']}>
        <Routes>
          <Route path="/troubleshoot/:category" element={<TroubleshootPage />} />
        </Routes>
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(troubleshootAPI.startSession).toHaveBeenCalledWith({
        tech_identifier: null,
        client_site: null,
        category: 'hardware',
      });
    });
  });
});
