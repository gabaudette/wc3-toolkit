import type { ReactNode } from "react";
import { Dialog, Heading, Modal, ModalOverlay } from "react-aria-components";
import { classes } from "../ui";

type DialogShellProps = {
  open: boolean;
  title: string;
  onClose: () => void;
  children: ReactNode;
  footer: ReactNode;
  wide?: boolean;
  page?: string;
  dismissable?: boolean;
};

export function DialogShell({
  open,
  title,
  onClose,
  children,
  footer,
  wide,
  page,
  dismissable = true,
}: DialogShellProps) {
  return (
    <ModalOverlay
      isOpen={open}
      onOpenChange={(isOpen) => {
        if (!isOpen) {
          onClose();
        }
      }}
      isDismissable={dismissable}
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/60 pt-[8vh]"
    >
      <Modal
        className={classes(
          "frame flex max-h-[85vh] max-w-[calc(100vw-2rem)] flex-col bg-surface-1 shadow-2xl",
          wide && "w-[64rem]",
          !wide && "w-[40rem]",
        )}
      >
        <Dialog className="flex min-h-0 flex-1 flex-col outline-none">
          <Heading
            slot="title"
            className="border-b border-line px-5 py-3 font-display text-2xl font-bold text-gold-bright"
          >
            {title}
          </Heading>
          <div
            key={page}
            className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto p-4 *:shrink-0"
          >
            {children}
          </div>
          <div className="flex justify-end gap-2 border-t border-line p-3">{footer}</div>
        </Dialog>
      </Modal>
    </ModalOverlay>
  );
}
