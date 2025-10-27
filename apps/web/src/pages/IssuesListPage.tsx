import { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { issuesAPI } from '../lib/api';
import type { Issue } from '../types/issues';
import IssueCard from '../components/IssueCard';
import TreeEditorModal from '../components/TreeEditorModal';
import CreateIssueModal from '../components/CreateIssueModal';
import ImportModal from '../components/ImportModal';
import { AccessibleConfirm } from '../components/AccessibleConfirm';
import { getErrorMessage } from '../lib/errorUtils';
import { logger } from '../lib/logger';

export default function IssuesListPage() {
  const navigate = useNavigate();
  const [issues, setIssues] = useState<Issue[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editingIssue, setEditingIssue] = useState<Issue | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showImportModal, setShowImportModal] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // Accessible confirm dialog state
  const [confirmDialog, setConfirmDialog] = useState<{ isOpen: boolean; title: string; message: string; onConfirm: () => void }>({
    isOpen: false,
    title: '',
    message: '',
    onConfirm: () => {},
  });

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
    } catch (err: unknown) {
      setError('Failed to load issues');
      logger.error('Failed to load issues list', { error: getErrorMessage(err) });
    } finally {
      setLoading(false);
    }
  };

  const handleToggle = useCallback(async (category: string, force = false) => {
    setError(null);
    try {
      const updatedIssue = await issuesAPI.toggle(category, force);
      setIssues(prevIssues => prevIssues.map(issue =>
        issue.category === category ? updatedIssue : issue
      ));
    } catch (err: unknown) {
      // Check if this is an axios error with validation data
      if (err && typeof err === 'object' && 'response' in err) {
        const axiosErr = err as { response?: { data?: { error?: { type?: string; data?: { fields?: { incomplete_nodes?: string }; message?: string } } } } };
        const validationError = axiosErr.response?.data?.error;
        if (validationError?.type === 'validation' && validationError?.data?.fields?.incomplete_nodes) {
          const message = validationError.data.fields.incomplete_nodes;
          setConfirmDialog({
            isOpen: true,
            title: 'Validation Warning',
            message: `${message}\n\nDo you want to activate this issue anyway?`,
            onConfirm: () => {
              // Retry with force=true
              handleToggle(category, true);
            },
          });
          return;
        }
      }
      setError(getErrorMessage(err) || 'Failed to toggle issue status. Please try again.');
      logger.error('Failed to toggle issue status', { category, error: getErrorMessage(err) });
    }
  }, []);

  const handleTest = useCallback((category: string) => {
    // Open the troubleshoot page with this category as the starting point
    window.open(`/?category=${category}`, '_blank');
  }, []);

  const handleEdit = useCallback((category: string) => {
    setIssues(prevIssues => {
      const issue = prevIssues.find(i => i.category === category);
      if (issue) {
        setEditingIssue(issue);
      }
      return prevIssues;
    });
  }, []);

  const handleDelete = useCallback(async (category: string, deleteSessions: boolean = false) => {
    setError(null);
    try {
      await issuesAPI.delete(category, deleteSessions);
      setIssues(prevIssues => prevIssues.filter(issue => issue.category !== category));
    } catch (err: unknown) {
      setError(getErrorMessage(err) || 'Failed to delete issue. Please try again.');
      logger.error('Failed to delete issue', { category, deleteSessions, error: getErrorMessage(err) });
    }
  }, []);

  const handleCreateNew = useCallback(() => {
    setShowCreateModal(true);
  }, []);

  const handleIssueCreated = useCallback((newIssue: Issue) => {
    setIssues(prevIssues => [...prevIssues, newIssue].sort((a, b) => a.name.localeCompare(b.name)));
  }, []);

  const handleExportAll = useCallback(async () => {
    setError(null);
    try {
      const data = await issuesAPI.exportAll();
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `issues-export-${new Date().toISOString().split('T')[0]}.json`;
      link.click();
      URL.revokeObjectURL(url);
      setSuccessMessage('All issues exported successfully!');
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err: unknown) {
      setError(getErrorMessage(err) || 'Failed to export issues. Please try again.');
      logger.error('Failed to export all issues', { error: getErrorMessage(err) });
    }
  }, []);

  const handleExportSingle = useCallback(async (category: string) => {
    setError(null);
    try {
      const data = await issuesAPI.exportIssue(category);
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `issue-${category}-${new Date().toISOString().split('T')[0]}.json`;
      link.click();
      URL.revokeObjectURL(url);
      setSuccessMessage(`Issue "${category}" exported successfully!`);
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err: unknown) {
      setError(getErrorMessage(err) || 'Failed to export issue. Please try again.');
      logger.error('Failed to export single issue', { category, error: getErrorMessage(err) });
    }
  }, []);

  const handleImportComplete = useCallback(() => {
    setShowImportModal(false);
    loadIssues(); // Reload issues after import
  }, []);

  const handleLogout = useCallback(() => {
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    navigate('/login');
  }, [navigate]);

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
            aria-label="Go to analytics dashboard"
          >
            📊 Analytics
          </button>
          <button
            onClick={() => navigate('/')}
            className="px-5 py-[10px] rounded-md bg-[#e0e0e0] text-gray-600 border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5"
            aria-label="View public troubleshooting site"
          >
            View Site
          </button>
          <button
            onClick={handleLogout}
            className="px-5 py-[10px] rounded-md bg-[#e0e0e0] text-gray-600 border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5"
            aria-label="Logout of admin panel"
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

        {successMessage && (
          <div className="mb-5 p-[15px] rounded-lg bg-[#efe] text-[#3c3] border border-[#cfc]">
            {successMessage}
          </div>
        )}

        <div className="flex justify-between items-center mb-5">
          <h2 className="text-[1.5em] font-bold text-gray-800 m-0">
            Troubleshooting Issues ({issues.length})
          </h2>
          <div className="flex gap-3">
            <button
              onClick={() => setShowImportModal(true)}
              className="px-5 py-[10px] rounded-md bg-[#10b981] text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5 font-medium"
              aria-label="Import issues from JSON file"
            >
              📥 Import
            </button>
            <button
              onClick={handleExportAll}
              className="px-5 py-[10px] rounded-md bg-[#3b82f6] text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5 font-medium"
              aria-label="Export all issues to JSON"
            >
              📤 Export All
            </button>
            <button
              onClick={handleCreateNew}
              className="px-5 py-[10px] rounded-md bg-gradient-to-br from-[#667eea] to-[#764ba2] text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5 font-medium"
              aria-label="Create a new troubleshooting issue"
            >
              + Create New Issue
            </button>
          </div>
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
                onExport={handleExportSingle}
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

      {/* Import Modal */}
      <ImportModal
        isOpen={showImportModal}
        onClose={() => setShowImportModal(false)}
        onSuccess={handleImportComplete}
      />

      {/* Accessible confirm dialog (replaces confirm()) */}
      <AccessibleConfirm
        isOpen={confirmDialog.isOpen}
        onClose={() => setConfirmDialog({ ...confirmDialog, isOpen: false })}
        onConfirm={confirmDialog.onConfirm}
        title={confirmDialog.title}
        message={confirmDialog.message}
      />
    </div>
  );
}
