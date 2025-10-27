import { useState, memo } from 'react';
import type { Issue } from '../types/issues';
import DeleteIssueModal from './DeleteIssueModal';

interface IssueCardProps {
  issue: Issue;
  onToggle: (category: string) => Promise<void>;
  onTest: (category: string) => void;
  onEdit: (category: string) => void;
  onDelete: (category: string, deleteSessions: boolean) => Promise<void>;
  onExport: (category: string) => Promise<void>;
}

const IssueCard = memo(function IssueCard({ issue, onToggle, onTest, onEdit, onDelete, onExport }: IssueCardProps) {
  const [toggling, setToggling] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);

  const handleToggle = async () => {
    setToggling(true);
    try {
      await onToggle(issue.category);
    } finally {
      setToggling(false);
    }
  };

  const handleDeleteClick = () => {
    setShowDeleteModal(true);
  };

  const handleDeleteConfirm = async (category: string, deleteSessions: boolean) => {
    await onDelete(category, deleteSessions);
  };

  return (
    <div className="border border-gray-200 rounded-lg p-5 mb-[15px] transition-shadow duration-200 hover:shadow-[0_4px_12px_rgba(0,0,0,0.1)]">
      <div className="flex justify-between items-start mb-[10px]">
        <div className="flex-1">
          <div className="flex items-center gap-3">
            <h3 className="font-semibold text-gray-800 text-[1.1em] m-0">
              {issue.name}
            </h3>
            {issue.display_category && (
              <span className="inline-block bg-blue-100 text-blue-700 px-3 py-1 rounded text-[0.85em] font-medium">
                {issue.display_category}
              </span>
            )}
          </div>
          <p className="text-gray-500 text-[0.9em] mt-2">
            {Number(issue.question_count)} question{Number(issue.question_count) !== 1 ? 's' : ''} in this decision tree
          </p>
        </div>

        {/* Toggle Switch */}
        <div className="flex items-center gap-2">
          <span className={`text-[0.85em] font-medium ${issue.is_active ? 'text-green-600' : 'text-gray-400'}`}>
            {issue.is_active ? 'Active' : 'Inactive'}
          </span>
          <button
            onClick={handleToggle}
            disabled={toggling}
            className={`w-12 h-6 rounded-full transition-colors duration-200 ${
              issue.is_active ? 'bg-green-500' : 'bg-gray-300'
            } ${toggling ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'} relative`}
            aria-label={`Turn ${issue.name} issue ${issue.is_active ? 'off' : 'on'}`}
            title={`Turn issue ${issue.is_active ? 'off' : 'on'}`}
          >
            <div
              className={`absolute top-0.5 w-5 h-5 bg-white rounded-full transition-transform duration-200 ${
                issue.is_active ? 'translate-x-6' : 'translate-x-0.5'
              }`}
            />
          </button>
        </div>
      </div>

      <div className="flex gap-[10px] mt-4">
        <button
          onClick={() => onEdit(issue.category)}
          className="px-3 py-[6px] text-[0.9em] rounded-md bg-gradient-to-br from-[#667eea] to-[#764ba2] text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5"
          aria-label={`Edit decision tree for ${issue.name}`}
        >
          ✏️ Edit Tree
        </button>
        <button
          onClick={() => onTest(issue.category)}
          className="px-3 py-[6px] text-[0.9em] rounded-md bg-[#4CAF50] text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5"
          aria-label={`Test troubleshooting flow for ${issue.name}`}
        >
          🧪 Test
        </button>
        <button
          onClick={() => onExport(issue.category)}
          className="px-3 py-[6px] text-[0.9em] rounded-md bg-[#3b82f6] text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5"
          aria-label={`Export ${issue.name} issue data`}
        >
          📤 Export
        </button>
        <button
          onClick={handleDeleteClick}
          className="px-3 py-[6px] text-[0.9em] rounded-md bg-[#f44336] text-white border-none transition-transform duration-200 hover:-translate-y-0.5 cursor-pointer"
          aria-label={`Delete ${issue.name} issue`}
        >
          🗑️ Delete
        </button>
      </div>

      {/* Delete Confirmation Modal */}
      <DeleteIssueModal
        issue={issue}
        isOpen={showDeleteModal}
        onClose={() => setShowDeleteModal(false)}
        onConfirm={handleDeleteConfirm}
      />
    </div>
  );
});

export default IssueCard;
