import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { questionsAPI } from '../lib/api';
import type { QuestionWithAnswers } from '../types';

export default function AdminDashboardPage() {
  const navigate = useNavigate();
  const [questions, setQuestions] = useState<QuestionWithAnswers[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadQuestions();
  }, []);

  const loadQuestions = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await questionsAPI.list();
      setQuestions(data);
    } catch (err: any) {
      setError('Failed to load questions');
      console.error('Error loading questions:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this question and all its answers?')) {
      return;
    }

    try {
      await questionsAPI.delete(id);
      setQuestions(questions.filter((q) => q.id !== id));
    } catch (err) {
      alert('Failed to delete question');
      console.error('Error deleting question:', err);
    }
  };

  const handleLogout = () => {
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    navigate('/admin/login');
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
          ⚙️ Admin Dashboard
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
            Questions ({questions.length})
          </h2>
          <button
            onClick={() => navigate('/admin/question/add')}
            className="px-5 py-[10px] rounded-md bg-gradient-to-br from-[#667eea] to-[#764ba2] text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5 font-medium"
          >
            + Add New Question
          </button>
        </div>

        {questions.length === 0 ? (
          <div className="text-center py-[60px_20px] text-gray-500">
            <h2 className="mb-[15px] text-gray-600 text-[1.3em]">No questions yet</h2>
            <p>Click "Add New Question" to get started!</p>
          </div>
        ) : (
          <div className="mt-5">
            {questions.map((question) => (
              <div
                key={question.id}
                className="border border-gray-200 rounded-lg p-5 mb-[15px] transition-shadow duration-200 hover:shadow-[0_4px_12px_rgba(0,0,0,0.1)]"
              >
                <div className="flex justify-between items-start mb-[10px]">
                  <div>
                    <span className="font-semibold text-[#667eea] text-[0.9em]">
                      {question.semantic_id}
                    </span>
                    {question.category && (
                      <span className="inline-block bg-[#f0f0f0] text-gray-600 px-3 py-1 rounded ml-[10px] text-[0.85em]">
                        {question.category}
                      </span>
                    )}
                  </div>
                </div>

                <div className="text-gray-800 mb-[15px] text-[1.05em]">
                  {question.text}
                </div>

                <div className="text-gray-500 text-[0.9em] mb-[15px]">
                  {question.answers?.length || 0} answer(s)
                </div>

                <div className="flex gap-[10px]">
                  <button
                    onClick={() => navigate(`/admin/question/${question.id}/edit`)}
                    className="px-3 py-[6px] text-[0.9em] rounded-md bg-gradient-to-br from-[#667eea] to-[#764ba2] text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5"
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => handleDelete(question.id)}
                    className="px-3 py-[6px] text-[0.9em] rounded-md bg-[#f44336] text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5"
                  >
                    Delete
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
