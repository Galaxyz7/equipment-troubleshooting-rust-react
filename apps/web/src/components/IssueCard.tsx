import { useState } from 'react';
import type { Issue } from '../types/issues';

interface IssueCardProps {
  issue: Issue;
  onToggle: (category: string) => Promise<void>;
  onTest: (category: string) => void;
  onEdit: (category: string) => void;
  onDelete: (category: string) => Promise<void>;
}

export default function IssueCard({ issue, onToggle, onTest, onEdit, onDelete }: IssueCardProps) {
  const [toggling, setToggling] = useState(false);
  const [deleting, setDeleting] = useState(false);

  const handleToggle = async () => {
    setToggling(true);
    try {
      await onToggle(issue.category);
    } finally {
      setToggling(false);
    }
  };

  const handleDelete = async () => {
    if (!confirm(`Are you sure you want to delete the issue "${issue.name}"? This will delete all ${issue.question_count} questions in this category and cannot be undone.`)) {
      return;
    }

    setDeleting(true);
    try {
      await onDelete(issue.category);
    } catch (error) {
      console.error('Error deleting issue:', error);
      setDeleting(false);
    }
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
            <span className="inline-block bg-[#f0f0f0] text-gray-600 px-3 py-1 rounded text-[0.85em]">
              {issue.category}
            </span>
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
        >
          ✏️ Edit Tree
        </button>
        <button
          onClick={() => onTest(issue.category)}
          className="px-3 py-[6px] text-[0.9em] rounded-md bg-[#4CAF50] text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5"
        >
          🧪 Test
        </button>
        <button
          onClick={handleDelete}
          disabled={deleting}
          className={`px-3 py-[6px] text-[0.9em] rounded-md bg-[#f44336] text-white border-none transition-transform duration-200 hover:-translate-y-0.5 ${
            deleting ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'
          }`}
        >
          {deleting ? '...' : '🗑️ Delete'}
        </button>
      </div>
    </div>
  );
}
