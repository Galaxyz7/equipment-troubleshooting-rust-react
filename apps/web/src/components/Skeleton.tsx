import { memo } from 'react';

interface SkeletonProps {
  /**
   * Width of the skeleton (can be any valid CSS width value)
   * @default '100%'
   */
  width?: string | number;

  /**
   * Height of the skeleton (can be any valid CSS height value)
   * @default '1rem'
   */
  height?: string | number;

  /**
   * Border radius of the skeleton
   * @default '0.375rem'
   */
  borderRadius?: string | number;

  /**
   * Additional CSS classes
   */
  className?: string;

  /**
   * Variant type for common skeleton patterns
   */
  variant?: 'text' | 'circular' | 'rectangular';

  /**
   * Number of skeleton lines to render (only for text variant)
   * @default 1
   */
  lines?: number;
}

/**
 * Skeleton loading component for improved perceived performance
 *
 * @example
 * // Single line skeleton
 * <Skeleton />
 *
 * @example
 * // Multiple text lines
 * <Skeleton variant="text" lines={3} />
 *
 * @example
 * // Circular avatar skeleton
 * <Skeleton variant="circular" width={48} height={48} />
 *
 * @example
 * // Custom rectangular skeleton
 * <Skeleton width="200px" height="120px" borderRadius="8px" />
 */
export const Skeleton = memo(function Skeleton({
  width = '100%',
  height = '1rem',
  borderRadius = '0.375rem',
  className = '',
  variant = 'text',
  lines = 1,
}: SkeletonProps) {
  const baseClasses = 'bg-gradient-to-r from-gray-200 via-gray-300 to-gray-200 bg-[length:200%_100%] animate-shimmer';

  const getVariantStyles = () => {
    switch (variant) {
      case 'circular':
        return { borderRadius: '50%' };
      case 'rectangular':
        return { borderRadius };
      case 'text':
      default:
        return { borderRadius };
    }
  };

  const style = {
    width: typeof width === 'number' ? `${width}px` : width,
    height: typeof height === 'number' ? `${height}px` : height,
    ...getVariantStyles(),
  };

  // Render multiple lines for text variant
  if (variant === 'text' && lines > 1) {
    return (
      <div className="space-y-2">
        {Array.from({ length: lines }).map((_, index) => {
          // Make last line slightly shorter for more natural appearance
          const isLastLine = index === lines - 1;
          const lineWidth = isLastLine ? '80%' : '100%';

          return (
            <div
              key={index}
              className={`${baseClasses} ${className}`}
              style={{ ...style, width: lineWidth }}
              role="status"
              aria-label="Loading..."
            />
          );
        })}
      </div>
    );
  }

  // Single skeleton
  return (
    <div
      className={`${baseClasses} ${className}`}
      style={style}
      role="status"
      aria-label="Loading..."
    />
  );
});

/**
 * Pre-configured skeleton for card loading
 */
export const SkeletonCard = memo(function SkeletonCard() {
  return (
    <div className="bg-white p-6 rounded-xl shadow">
      <div className="flex items-center gap-4 mb-4">
        <Skeleton variant="circular" width={48} height={48} />
        <div className="flex-1">
          <Skeleton width="60%" height="1.25rem" className="mb-2" />
          <Skeleton width="40%" height="0.875rem" />
        </div>
      </div>
      <Skeleton variant="text" lines={3} />
      <div className="mt-4 flex gap-2">
        <Skeleton width={80} height={36} borderRadius="0.5rem" />
        <Skeleton width={80} height={36} borderRadius="0.5rem" />
      </div>
    </div>
  );
});

/**
 * Pre-configured skeleton for table rows
 */
export const SkeletonTable = memo(function SkeletonTable({ rows = 5 }: { rows?: number }) {
  return (
    <div className="space-y-3">
      {Array.from({ length: rows }).map((_, index) => (
        <div key={index} className="flex gap-4 items-center">
          <Skeleton width={24} height={24} />
          <Skeleton width="30%" height="1rem" />
          <Skeleton width="20%" height="1rem" />
          <Skeleton width="25%" height="1rem" />
          <Skeleton width="15%" height="1rem" />
        </div>
      ))}
    </div>
  );
});

/**
 * Pre-configured skeleton for form fields
 */
export const SkeletonForm = memo(function SkeletonForm({ fields = 3 }: { fields?: number }) {
  return (
    <div className="space-y-4">
      {Array.from({ length: fields }).map((_, index) => (
        <div key={index}>
          <Skeleton width="30%" height="0.875rem" className="mb-2" />
          <Skeleton width="100%" height="2.5rem" borderRadius="0.5rem" />
        </div>
      ))}
      <div className="flex gap-2 mt-6">
        <Skeleton width={100} height={40} borderRadius="0.5rem" />
        <Skeleton width={100} height={40} borderRadius="0.5rem" />
      </div>
    </div>
  );
});
