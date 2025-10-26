import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import IssuesListPage from './IssuesListPage';
import type { Issue } from '../types/issues';

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
  issuesAPI: {
    list: vi.fn(),
    toggle: vi.fn(),
    delete: vi.fn(),
    create: vi.fn(),
  },
}));

// Mock TreeEditorModal component
vi.mock('../components/TreeEditorModal', () => ({
  default: () => <div data-testid="tree-editor-modal">Tree Editor Modal</div>,
}));

import { issuesAPI } from '../lib/api';

/**
 * IssuesListPage Component Tests
 * Tests issue listing, CRUD operations, and management
 */

const mockIssues: Issue[] = [
  {
    id: '1',
    name: 'Hardware Issues',
    category: 'hardware',
    display_category: 'Hardware',
    root_question_id: 'q1',
    is_active: true,
    question_count: 10n,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    id: '2',
    name: 'Software Issues',
    category: 'software',
    display_category: 'Software',
    root_question_id: 'q2',
    is_active: false,
    question_count: 5n,
    created_at: '2024-01-02T00:00:00Z',
    updated_at: '2024-01-02T00:00:00Z',
  },
];

describe('IssuesListPage', () => {
  beforeEach(() => {
    mockNavigate.mockClear();
    vi.mocked(issuesAPI.list).mockClear();
    vi.mocked(issuesAPI.toggle).mockClear();
    vi.mocked(issuesAPI.delete).mockClear();
    vi.mocked(issuesAPI.create).mockClear();
  });

  it('should load and display issues on mount', async () => {
    vi.mocked(issuesAPI.list).mockResolvedValue(mockIssues);

    render(
      <MemoryRouter>
        <IssuesListPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(issuesAPI.list).toHaveBeenCalled();
    });

    await waitFor(() => {
      expect(screen.getByText('Hardware Issues')).toBeInTheDocument();
      expect(screen.getByText('Software Issues')).toBeInTheDocument();
    });
  });

  it('should display loading state while fetching issues', async () => {
    const listPromise = new Promise(() => {
      // Never resolves to keep loading state
    });

    vi.mocked(issuesAPI.list).mockReturnValue(listPromise as any);

    render(
      <MemoryRouter>
        <IssuesListPage />
      </MemoryRouter>
    );

    expect(screen.getByText(/Loading/i)).toBeInTheDocument();
  });

  it('should display error message when loading fails', async () => {
    vi.mocked(issuesAPI.list).mockRejectedValue(new Error('Network error'));

    render(
      <MemoryRouter>
        <IssuesListPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(screen.getByText(/Failed to load issues/i)).toBeInTheDocument();
    });
  });

  it('should filter out internal categories', async () => {
    const issuesWithInternal: Issue[] = [
      ...mockIssues,
      {
        id: '3',
        name: 'Root',
        category: 'root',
        display_category: null,
        root_question_id: 'q3',
        is_active: true,
        question_count: 1n,
        created_at: '2024-01-03T00:00:00Z',
        updated_at: '2024-01-03T00:00:00Z',
      },
    ];

    vi.mocked(issuesAPI.list).mockResolvedValue(issuesWithInternal);

    render(
      <MemoryRouter>
        <IssuesListPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(screen.getByText('Hardware Issues')).toBeInTheDocument();
    });

    // Root category should not be displayed
    expect(screen.queryByText('Root')).not.toBeInTheDocument();
  });

  it('should display Create New Issue button', async () => {
    vi.mocked(issuesAPI.list).mockResolvedValue(mockIssues);

    render(
      <MemoryRouter>
        <IssuesListPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /Create New Issue/i })).toBeInTheDocument();
    });
  });

  it('should render IssueCard for each issue', async () => {
    vi.mocked(issuesAPI.list).mockResolvedValue(mockIssues);

    render(
      <MemoryRouter>
        <IssuesListPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      // Check for issue-specific elements
      expect(screen.getByText('10 questions in this decision tree')).toBeInTheDocument();
      expect(screen.getByText('5 questions in this decision tree')).toBeInTheDocument();
    });
  });

  it('should call toggle API when issue is toggled', async () => {
    const user = userEvent.setup();
    const updatedIssue = { ...mockIssues[0], is_active: false };

    vi.mocked(issuesAPI.list).mockResolvedValue(mockIssues);
    vi.mocked(issuesAPI.toggle).mockResolvedValue(updatedIssue);

    render(
      <MemoryRouter>
        <IssuesListPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(screen.getByText('Hardware Issues')).toBeInTheDocument();
    });

    // Find and click the toggle button for Hardware Issues
    const toggleButtons = screen.getAllByTitle(/Turn issue/i);
    await user.click(toggleButtons[0]);

    await waitFor(() => {
      expect(issuesAPI.toggle).toHaveBeenCalledWith('hardware', false);
    });
  });

  it('should handle delete operation', async () => {
    const user = userEvent.setup();

    vi.mocked(issuesAPI.list).mockResolvedValue(mockIssues);
    vi.mocked(issuesAPI.delete).mockResolvedValue(undefined);

    // Mock window.confirm
    vi.stubGlobal('confirm', vi.fn(() => true));

    render(
      <MemoryRouter>
        <IssuesListPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(screen.getByText('Hardware Issues')).toBeInTheDocument();
    });

    // Find and click delete button
    const deleteButtons = screen.getAllByText(/Delete/i);
    await user.click(deleteButtons[0]);

    await waitFor(() => {
      expect(issuesAPI.delete).toHaveBeenCalledWith('hardware');
    });

    // Issue should be removed from list
    await waitFor(() => {
      expect(screen.queryByText('Hardware Issues')).not.toBeInTheDocument();
    });

    vi.unstubAllGlobals();
  });

  it('should open TreeEditorModal when edit is clicked', async () => {
    const user = userEvent.setup();

    vi.mocked(issuesAPI.list).mockResolvedValue(mockIssues);

    render(
      <MemoryRouter>
        <IssuesListPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(screen.getByText('Hardware Issues')).toBeInTheDocument();
    });

    // Find and click edit button
    const editButtons = screen.getAllByText(/Edit Tree/i);
    await user.click(editButtons[0]);

    // Tree editor modal should appear
    await waitFor(() => {
      expect(screen.getByTestId('tree-editor-modal')).toBeInTheDocument();
    });
  });

  it('should open test page in new window when test is clicked', async () => {
    const user = userEvent.setup();

    vi.mocked(issuesAPI.list).mockResolvedValue(mockIssues);

    // Mock window.open
    const mockWindowOpen = vi.fn();
    vi.stubGlobal('open', mockWindowOpen);

    render(
      <MemoryRouter>
        <IssuesListPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(screen.getByText('Hardware Issues')).toBeInTheDocument();
    });

    // Find and click test button
    const testButtons = screen.getAllByText(/Test/i);
    await user.click(testButtons[0]);

    expect(mockWindowOpen).toHaveBeenCalledWith('/?category=hardware', '_blank');

    vi.unstubAllGlobals();
  });

  it('should sort issues alphabetically by name', async () => {
    const unsortedIssues: Issue[] = [
      { ...mockIssues[1], name: 'Zebra Issues' },
      { ...mockIssues[0], name: 'Apple Issues' },
      { ...mockIssues[0], name: 'Banana Issues' },
    ];

    vi.mocked(issuesAPI.list).mockResolvedValue(unsortedIssues);

    render(
      <MemoryRouter>
        <IssuesListPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      const issueNames = screen.getAllByText(/Issues$/);
      expect(issueNames[0]).toHaveTextContent('Apple Issues');
      expect(issueNames[1]).toHaveTextContent('Banana Issues');
      expect(issueNames[2]).toHaveTextContent('Zebra Issues');
    });
  });

  it('should display empty state when no issues exist', async () => {
    vi.mocked(issuesAPI.list).mockResolvedValue([]);

    render(
      <MemoryRouter>
        <IssuesListPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      // Should still show create button
      expect(screen.getByRole('button', { name: /Create New Issue/i })).toBeInTheDocument();
    });

    // No issue cards should be displayed
    expect(screen.queryByText(/questions in this decision tree/)).not.toBeInTheDocument();
  });
});
