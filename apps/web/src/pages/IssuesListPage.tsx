import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { issuesAPI } from '../lib/api';
import type { Issue } from '../types/issues';
import IssueCard from '../components/IssueCard';
import TreeEditorModal from '../components/TreeEditorModal';
import CreateIssueModal from '../components/CreateIssueModal';

export default function IssuesListPage() {
  const navigate = useNavigate();
  const [issues, setIssues] = useState<Issue[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editingIssue, setEditingIssue] = useState<Issue | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);

  useEffect(() => {
    loadIssues();
  }, []);

  const loadIssues = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await issuesAPI.list();
      // Filter out:
      // - 'root' category (contains starting question, not a user-facing issue)
      // - Sub-categories like 'electrical', 'general', 'mechanical' (shared questions used within other trees)
      // Only show top-level user-facing issues
      const subCategories = ['root', 'electrical', 'general', 'mechanical'];
      setIssues(data
        .filter(issue => !subCategories.includes(issue.category))
        .sort((a, b) => a.name.localeCompare(b.name))
      );
    } catch (err: any) {
      setError('Failed to load issues');
      console.error('Error loading issues:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleToggle = async (category: string, force = false) => {
    setError(null);
    try {
      const updatedIssue = await issuesAPI.toggle(category, force);
      setIssues(issues.map(issue =>
        issue.category === category ? updatedIssue : issue
      ));
    } catch (err: any) {
      // Check if this is a validation error about incomplete nodes
      const validationError = err.response?.data?.error;
      if (validationError?.type === 'validation' && validationError?.data?.fields?.incomplete_nodes) {
        const message = validationError.data.fields.incomplete_nodes;
        const confirmed = confirm(
          `${message}\n\nDo you want to activate this issue anyway?`
        );
        if (confirmed) {
          // Retry with force=true
          await handleToggle(category, true);
        }
      } else {
        const errorMessage = err.response?.data?.error?.data?.message ||
                            'Failed to toggle issue status. Please try again.';
        setError(errorMessage);
        console.error('Error toggling issue:', err);
      }
    }
  };

  const handleTest = (category: string) => {
    // Open the troubleshoot page with this category as the starting point
    window.open(`/?category=${category}`, '_blank');
  };

  const handleEdit = (category: string) => {
    const issue = issues.find(i => i.category === category);
    if (issue) {
      setEditingIssue(issue);
    }
  };

  const handleDelete = async (category: string) => {
    setError(null);
    try {
      await issuesAPI.delete(category);
      setIssues(issues.filter(issue => issue.category !== category));
    } catch (err: any) {
      const errorMessage = err.response?.data?.error?.data?.message ||
                          'Failed to delete issue. Please try again.';
      setError(errorMessage);
      console.error('Error deleting issue:', err);
    }
  };

  const handleCreateNew = () => {
    setShowCreateModal(true);
  };

  const handleIssueCreated = (newIssue: Issue) => {
    setIssues([...issues, newIssue].sort((a, b) => a.name.localeCompare(b.name)));
  };

  const handleLogout = () => {
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    navigate('/login');
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-[#f5f5f5] flex items-center justify-center">
        <div className="text-2xl font-semibold text-gray-700">Loading...</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-[#f5f5f5] p-5">
      {/* Header */}
      <div className="bg-white p-[20px_30px] rounded-xl mb-[30px] shadow-[0_2px_10px_rgba(0,0,0,0.1)] flex justify-between items-center">
        <h1 className="text-[2em] font-bold text-gray-800 m-0">
          ⚙️ Issues Management
        </h1>
        <div className="flex gap-[15px]">
          <button
            onClick={() => navigate('/admin/analytics')}
            className="px-5 py-[10px] rounded-md bg-gradient-to-br from-[#667eea] to-[#764ba2] text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5"
          >
            📊 Analytics
          </button>
          <button
            onClick={() => navigate('/')}
            className="px-5 py-[10px] rounded-md bg-[#e0e0e0] text-gray-600 border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5"
          >
            View Site
          </button>
          <button
            onClick={handleLogout}
            className="px-5 py-[10px] rounded-md bg-[#e0e0e0] text-gray-600 border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5"
          >
            Logout
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="bg-white p-[30px] rounded-xl shadow-[0_2px_10px_rgba(0,0,0,0.1)]">
        {error && (
          <div className="mb-5 p-[15px] rounded-lg bg-[#fee] text-[#c33] border border-[#fcc]">
            {error}
          </div>
        )}

        <div className="flex justify-between items-center mb-5">
          <h2 className="text-[1.5em] font-bold text-gray-800 m-0">
            Troubleshooting Issues ({issues.length})
          </h2>
          <button
            onClick={handleCreateNew}
            className="px-5 py-[10px] rounded-md bg-gradient-to-br from-[#667eea] to-[#764ba2] text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5 font-medium"
          >
            + Create New Issue
          </button>
        </div>

        <div className="mb-5 p-4 bg-blue-50 border border-blue-200 rounded-lg">
          <p className="text-sm text-gray-700 m-0">
            <strong>What are Issues?</strong> Each issue represents a top-level troubleshooting category with its own decision tree.
            Users start by selecting an issue, then answer questions to navigate through the decision tree.
          </p>
        </div>

        {issues.length === 0 ? (
          <div className="text-center py-[60px_20px] text-gray-500">
            <h2 className="mb-[15px] text-gray-600 text-[1.3em]">No issues yet</h2>
            <p>Click "Create New Issue" to add your first troubleshooting category!</p>
          </div>
        ) : (
          <div className="mt-5">
            {issues.map((issue) => (
              <IssueCard
                key={issue.id}
                issue={issue}
                onToggle={handleToggle}
                onTest={handleTest}
                onEdit={handleEdit}
                onDelete={handleDelete}
              />
            ))}
          </div>
        )}
      </div>

      {/* Tree Editor Modal */}
      {editingIssue && (
        <TreeEditorModal
          category={editingIssue.category}
          issueName={editingIssue.name}
          onClose={() => setEditingIssue(null)}
          onSave={() => {
            setEditingIssue(null);
            loadIssues(); // Reload issues after save
          }}
        />
      )}

      {/* Create Issue Modal */}
      <CreateIssueModal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        onCreate={handleIssueCreated}
      />
    </div>
  );
}
