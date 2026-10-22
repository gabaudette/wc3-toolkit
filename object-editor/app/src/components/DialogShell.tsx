import type { ReactNode } from "react";
import { Dialog, Heading, Modal, ModalOverlay } from "react-aria-components";

type DialogShellProps = {
  open: boolean;
  title: string;
  onClose: () => void;
  children: ReactNode;
  /** Buttons along the bottom edge. */
  footer: ReactNode;
};

/** A centered dialog with a title, scrolling content and a footer. */
export function DialogShell({ open, title, onClose, children, footer }: DialogShellProps) {
  return (
    <ModalOverlay
      isOpen={open}
      onOpenChange={(isOpen) => {
        if (!isOpen) {
          onClose();
        }
      }}
      isDismissable
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
    >
      <Modal className="flex max-h-[85vh] w-[40rem] flex-col rounded-lg border border-line bg-surface-1 shadow-2xl">
        <Dialog className="flex min-h-0 flex-1 flex-col outline-none">
          <Heading slot="title" className="border-b border-line p-4 text-lg font-semibold">
            {title}
          </Heading>
          {/* Parts keep their height; the whole body scrolls instead. */}
          <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto p-4 *:shrink-0">
            {children}
          </div>
          <div className="flex justify-end gap-2 border-t border-line p-3">{footer}</div>
        </Dialog>
      </Modal>
    </ModalOverlay>
  );
}
