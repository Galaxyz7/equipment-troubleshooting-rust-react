import { useState, useEffect } from 'react';
import { adminAPI } from '../lib/api';
import { getErrorMessage } from '../lib/errorUtils';
import { logger } from '../lib/logger';

interface CategoryComboboxProps {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  placeholder?: string;
  className?: string;
  label?: string;
  optional?: boolean;
  description?: string;
}

export default function CategoryCombobox({
  value,
  onChange,
  disabled = false,
  placeholder = 'Select or type a category',
  className = '',
  label,
  optional = false,
  description,
}: CategoryComboboxProps) {
  const [categories, setCategories] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadCategories();
  }, []);

  const loadCategories = async () => {
    try {
      const data = await adminAPI.getCategories();
      setCategories(data.categories);
    } catch (err) {
      logger.error('Failed to load categories for combobox', { error: getErrorMessage(err) });
      setCategories([]);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      {label && (
        <label className="block text-sm font-medium text-gray-700 mb-2">
          {label} {optional && <span className="text-gray-400">(optional)</span>}
        </label>
      )}
      <div className="relative">
        <input
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          disabled={disabled || loading}
          placeholder={loading ? 'Loading categories...' : placeholder}
          className={`w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent transition-colors ${className}`}
          list="category-options"
          autoComplete="off"
        />
        <datalist id="category-options">
          {categories.map((cat) => (
            <option key={cat} value={cat} />
          ))}
        </datalist>
      </div>
      {description && (
        <p className="text-gray-500 text-sm mt-1 flex items-center">
          <span className="mr-1">💡</span>
          {description}
        </p>
      )}
    </div>
  );
}
