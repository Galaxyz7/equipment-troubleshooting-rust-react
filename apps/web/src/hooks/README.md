# Custom React Hooks

This directory contains reusable custom hooks that encapsulate common patterns and state logic across the application.

## Available Hooks

### `useToast`

Manages toast/alert notification state for displaying success, error, and info messages.

**Usage:**
```tsx
import { useToast } from '../hooks/useToast';

function MyComponent() {
  const { toastState, showToast, hideToast } = useToast();

  const handleSuccess = () => {
    showToast({
      title: 'Success!',
      message: 'Operation completed successfully',
      type: 'success'
    });
  };

  return (
    <>
      <button onClick={handleSuccess}>Do Something</button>
      <AccessibleAlert
        isOpen={toastState.isOpen}
        title={toastState.title}
        message={toastState.message}
        type={toastState.type}
        onClose={hideToast}
      />
    </>
  );
}
```

**Benefits:**
- Eliminates repetitive alert state management (`useState` for isOpen, title, message, type)
- Provides consistent interface across all components
- Easy to extend with auto-dismiss functionality

---

### `useConfirm`

Manages confirmation dialog state for dangerous operations (deletes, overwrites, etc.).

**Usage:**
```tsx
import { useConfirm } from '../hooks/useConfirm';

function MyComponent() {
  const { confirmState, showConfirm, hideConfirm, handleConfirm } = useConfirm();

  const handleDelete = () => {
    showConfirm({
      title: 'Delete Item',
      message: 'Are you sure? This action cannot be undone.',
      variant: 'danger',
      onConfirm: async () => {
        await api.delete('/items/123');
        console.log('Deleted!');
      }
    });
  };

  return (
    <>
      <button onClick={handleDelete}>Delete</button>
      <AccessibleConfirm
        isOpen={confirmState.isOpen}
        title={confirmState.title}
        message={confirmState.message}
        variant={confirmState.variant}
        onConfirm={handleConfirm}
        onCancel={hideConfirm}
      />
    </>
  );
}
```

**Benefits:**
- Eliminates repetitive confirm dialog state management
- Encapsulates async operation handling
- Consistent UX for dangerous operations

---

## Hook Architecture Principles

1. **Single Responsibility**: Each hook manages one specific piece of state logic
2. **Reusability**: Hooks are designed to work across multiple components
3. **Type Safety**: Full TypeScript support with exported interfaces
4. **Performance**: Uses `useCallback` to prevent unnecessary re-renders
5. **Documentation**: Comprehensive JSDoc comments for IDE autocomplete

## Future Hooks

Potential hooks to consider adding:

- `useDebounce` - Debounce user input for search/filter operations
- `useLocalStorage` - Sync state with localStorage
- `useMediaQuery` - Responsive design utilities
- `useApi` - Standardized API call wrapper with loading/error states
- `useForm` - Form state management and validation

## Migration Guide

### Before (Inline State):
```tsx
const [alertDialog, setAlertDialog] = useState({
  isOpen: false,
  title: '',
  message: '',
  type: 'info'
});

const showSuccess = () => {
  setAlertDialog({
    isOpen: true,
    title: 'Success',
    message: 'Saved!',
    type: 'success'
  });
};
```

### After (useToast Hook):
```tsx
const { toastState, showToast } = useToast();

const showSuccess = () => {
  showToast({
    title: 'Success',
    message: 'Saved!',
    type: 'success'
  });
};
```

**Result**: ~10 lines → 3 lines, better reusability, consistent patterns
