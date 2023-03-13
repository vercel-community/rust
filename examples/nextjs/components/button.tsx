import { clsx } from 'clsx';
import React from 'react';

type Props = Omit<React.ComponentProps<'button'>, 'className'> & {
  loading?: boolean;
};

const Button = React.forwardRef<HTMLButtonElement, Props>(
  ({ children, loading = false, ...props }, ref) => (
    <button
      disabled={loading}
      ref={ref}
      {...props}
      className={clsx(
        loading ? 'cursor-not-allowed' : 'hover:bg-gray-700',
        'relative inline-flex select-none items-center justify-center rounded-md px-4 py-2 text-sm font-medium',
        'bg-gray-800 text-gray-100',
        'focus:outline-none focus-visible:ring focus-visible:ring-purple-500 focus-visible:ring-opacity-75',
        // Register all radix states
        'group',
        'radix-state-open:bg-gray-700',
        'radix-state-on:bg-gray-700',
        'radix-state-instant-open:bg-gray-50 radix-state-delayed-open:bg-gray-50',
      )}
    >
      {loading && (
        <div className="absolute inset-0 inline-flex items-center justify-center bg-gray-800 rounded-md">
          <svg
            className="animate-spin h-5 w-5 text-white"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle
              className="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              strokeWidth="4"
            />
            <path
              className="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
        </div>
      )}

      {children}
    </button>
  ),
);

Button.displayName = 'Button';
export default Button;
