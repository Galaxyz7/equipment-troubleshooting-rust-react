import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { adminAPI } from '../lib/api';
import type { DashboardStats, CategoryStats, ConclusionStats } from '../types/troubleshoot';
import DataManagementModal from '../components/DataManagementModal';

export default function AnalyticsPage() {
  const navigate = useNavigate();
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [showDataManagement, setShowDataManagement] = useState(false);
  const [successMessage, setSuccessMessage] = useState('');

  useEffect(() => {
    loadStats();
  }, []);

  const loadStats = async () => {
    try {
      const data = await adminAPI.getStats();
      setStats(data);
      setError('');
    } catch (err: any) {
      setError(err.response?.data?.error?.data?.message || 'Failed to load statistics');
    } finally {
      setLoading(false);
    }
  };

  const handleDataManagementSuccess = () => {
    setSuccessMessage('Sessions deleted successfully');
    loadStats(); // Refresh stats
    setTimeout(() => setSuccessMessage(''), 5000);
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 p-6 flex items-center justify-center">
        <div className="text-center">
          <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-purple-600 border-r-transparent"></div>
          <p className="mt-3 text-gray-600">Loading analytics...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-gray-50 p-6">
        <div className="max-w-7xl mx-auto">
          <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
            {error}
          </div>
        </div>
      </div>
    );
  }

  if (!stats) {
    return null;
  }

  const completionRate = stats.total_sessions > 0
    ? ((Number(stats.completed_sessions) / Number(stats.total_sessions)) * 100).toFixed(1)
    : '0';

  const abandonmentRate = stats.total_sessions > 0
    ? ((Number(stats.abandoned_sessions) / Number(stats.total_sessions)) * 100).toFixed(1)
    : '0';

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div>
              <button
                onClick={() => navigate('/admin')}
                className="text-purple-600 hover:text-purple-700 mb-2 inline-flex items-center text-sm"
              >
                ← Back to Issues
              </button>
              <h1 className="text-3xl font-bold text-gray-900">Analytics Dashboard</h1>
              <p className="text-gray-600 mt-1">Troubleshooting session insights</p>
            </div>
            <button
              onClick={() => setShowDataManagement(true)}
              className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 font-medium inline-flex items-center"
            >
              <span className="mr-2">🗑️</span>
              Data Management
            </button>
          </div>
        </div>
      </div>

      {/* Success Message */}
      {successMessage && (
        <div className="max-w-7xl mx-auto px-6 pt-6">
          <div className="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg">
            {successMessage}
          </div>
        </div>
      )}

      {/* Stats Grid */}
      <div className="max-w-7xl mx-auto px-6 py-8">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <StatCard
            title="Total Sessions"
            value={Number(stats.total_sessions).toLocaleString()}
            icon="📊"
            color="blue"
            subtitle="All time"
          />
          <StatCard
            title="Completed"
            value={Number(stats.completed_sessions).toLocaleString()}
            icon="✅"
            color="green"
            subtitle={`${completionRate}% completion rate`}
          />
          <StatCard
            title="Abandoned"
            value={Number(stats.abandoned_sessions).toLocaleString()}
            icon="⚠️"
            color="yellow"
            subtitle={`${abandonmentRate}% abandoned`}
          />
          <StatCard
            title="Avg Steps"
            value={stats.avg_steps_to_completion.toFixed(1)}
            icon="👣"
            color="purple"
            subtitle="To completion"
          />
        </div>

        {/* Charts Row */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
          {/* Most Common Conclusions */}
          <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
            <h2 className="text-xl font-bold text-gray-900 mb-4">Most Common Conclusions</h2>
            {stats.most_common_conclusions.length > 0 ? (
              <div className="space-y-3">
                {stats.most_common_conclusions.map((conclusion, index) => (
                  <ConclusionBar
                    key={index}
                    conclusion={conclusion}
                    maxCount={Number(stats.most_common_conclusions[0].count)}
                    rank={index + 1}
                  />
                ))}
              </div>
            ) : (
              <p className="text-gray-500 text-center py-8">No conclusions data yet</p>
            )}
          </div>

          {/* Sessions by Category */}
          <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
            <h2 className="text-xl font-bold text-gray-900 mb-4">Sessions by Category</h2>
            {stats.sessions_by_category.length > 0 ? (
              <div className="space-y-3">
                {stats.sessions_by_category.map((category, index) => (
                  <CategoryBar
                    key={index}
                    category={category}
                    maxCount={Number(stats.sessions_by_category[0].count)}
                    rank={index + 1}
                  />
                ))}
              </div>
            ) : (
              <p className="text-gray-500 text-center py-8">No category data yet</p>
            )}
          </div>
        </div>

        {/* Active Sessions */}
        {stats.active_sessions > 0 && (
          <div className="bg-gradient-to-r from-blue-50 to-purple-50 border border-blue-200 rounded-xl p-6">
            <div className="flex items-center">
              <div className="text-4xl mr-4">🔄</div>
              <div>
                <h3 className="text-lg font-semibold text-gray-900">
                  {Number(stats.active_sessions)} Active Session{stats.active_sessions !== 1n ? 's' : ''}
                </h3>
                <p className="text-gray-600">
                  Session{stats.active_sessions !== 1n ? 's' : ''} in progress (not completed or abandoned)
                </p>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Data Management Modal */}
      <DataManagementModal
        isOpen={showDataManagement}
        onClose={() => setShowDataManagement(false)}
        onSuccess={handleDataManagementSuccess}
      />
    </div>
  );
}

// Stat Card Component
interface StatCardProps {
  title: string;
  value: string;
  icon: string;
  color: 'blue' | 'green' | 'yellow' | 'purple';
  subtitle?: string;
}

function StatCard({ title, value, icon, color, subtitle }: StatCardProps) {
  const colors = {
    blue: 'bg-blue-50 text-blue-700',
    green: 'bg-green-50 text-green-700',
    yellow: 'bg-yellow-50 text-yellow-700',
    purple: 'bg-purple-50 text-purple-700',
  };

  return (
    <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200 hover:shadow-md transition-shadow">
      <div className="flex items-center justify-between mb-2">
        <p className="text-sm font-medium text-gray-600">{title}</p>
        <div className={`text-2xl ${colors[color]} w-12 h-12 rounded-full flex items-center justify-center`}>
          {icon}
        </div>
      </div>
      <p className="text-3xl font-bold text-gray-900">{value}</p>
      {subtitle && <p className="text-sm text-gray-500 mt-1">{subtitle}</p>}
    </div>
  );
}

// Conclusion Bar Component
interface ConclusionBarProps {
  conclusion: ConclusionStats;
  maxCount: number;
  rank: number;
}

function ConclusionBar({ conclusion, maxCount, rank }: ConclusionBarProps) {
  const percentage = (Number(conclusion.count) / maxCount) * 100;

  return (
    <div className="group">
      <div className="flex items-center justify-between text-sm mb-1">
        <span className="font-medium text-gray-700 flex items-center">
          <span className="inline-flex items-center justify-center w-6 h-6 rounded-full bg-gray-100 text-gray-600 text-xs font-bold mr-2">
            {rank}
          </span>
          {conclusion.conclusion}
        </span>
        <span className="text-gray-500">{Number(conclusion.count)}</span>
      </div>
      <div className="w-full bg-gray-200 rounded-full h-2 overflow-hidden">
        <div
          className="bg-gradient-to-r from-purple-500 to-blue-500 h-2 rounded-full transition-all duration-300 group-hover:from-purple-600 group-hover:to-blue-600"
          style={{ width: `${percentage}%` }}
        />
      </div>
    </div>
  );
}

// Category Bar Component
interface CategoryBarProps {
  category: CategoryStats;
  maxCount: number;
  rank: number;
}

function CategoryBar({ category, maxCount, rank }: CategoryBarProps) {
  const percentage = (Number(category.count) / maxCount) * 100;

  return (
    <div className="group">
      <div className="flex items-center justify-between text-sm mb-1">
        <span className="font-medium text-gray-700 flex items-center">
          <span className="inline-flex items-center justify-center w-6 h-6 rounded-full bg-gray-100 text-gray-600 text-xs font-bold mr-2">
            {rank}
          </span>
          {category.category}
        </span>
        <span className="text-gray-500">{Number(category.count)}</span>
      </div>
      <div className="w-full bg-gray-200 rounded-full h-2 overflow-hidden">
        <div
          className="bg-gradient-to-r from-green-500 to-emerald-500 h-2 rounded-full transition-all duration-300 group-hover:from-green-600 group-hover:to-emerald-600"
          style={{ width: `${percentage}%` }}
        />
      </div>
    </div>
  );
}
