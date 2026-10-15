import * as DialogPrimitive from '@radix-ui/react-dialog';
import { X } from 'lucide-react';
import type { ReactNode } from 'react';
export function Dialog({
  title,
  description,
  children,
  onClose,
  busy = false,
  className = '',
  onReturnFocus,
}: {
  title: string;
  description: string;
  children: ReactNode;
  onClose: () => void;
  busy?: boolean;
  className?: string;
  onReturnFocus?: () => void;
}) {
  return (
    <DialogPrimitive.Root
      open
      onOpenChange={(open) => {
        if (!open && !busy) onClose();
      }}
    >
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay className="dialog-overlay" />
        <DialogPrimitive.Content
          className={'dialog-content ' + className}
          onCloseAutoFocus={(e) => {
            if (onReturnFocus) {
              e.preventDefault();
              // Let the caller re-enable controls after an async mutation.
              setTimeout(onReturnFocus, 0);
            }
          }}
          onEscapeKeyDown={(e) => {
            if (busy) e.preventDefault();
          }}
          onPointerDownOutside={(e) => e.preventDefault()}
        >
          <div className="dialog-heading">
            <DialogPrimitive.Title>{title}</DialogPrimitive.Title>
            <DialogPrimitive.Close
              className="icon-button"
              aria-label="Close dialog"
              disabled={busy}
            >
              <X size={18} />
            </DialogPrimitive.Close>
          </div>
          <DialogPrimitive.Description className="muted dialog-description">
            {description}
          </DialogPrimitive.Description>
          {children}
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  );
}
