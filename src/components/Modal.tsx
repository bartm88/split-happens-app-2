import { ReactNode } from 'react';

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  children: ReactNode;
  title?: string;
}

export const Modal = ({ isOpen, onClose, children, title }: ModalProps) => {
  if (!isOpen) return null;

  return (
    <>
      {/* Backdrop */}
      <div 
        className="fixed inset-0 z-40"
        style={{ backgroundColor: 'rgba(0, 0, 0, 0.6)' }}
        onClick={onClose}
      />
      
      {/* Modal */}
      <div className="fixed inset-0 flex items-center justify-center z-50 p-4 pointer-events-none">
        <div className="bg-neutral rounded-lg shadow-2xl max-w-md w-full max-h-[90vh] overflow-auto border border-neutral pointer-events-auto">
          {title && (
            <div className="flex justify-between items-center p-4 border-b border-neutral">
              <h2 className="text-xl font-semibold text-neutral">{title}</h2>
              <button
                onClick={onClose}
                className="text-neutral hover:text-negative transition-colors text-2xl leading-none"
              >
                ×
              </button>
            </div>
          )}
          <div className="p-6">
            {children}
          </div>
        </div>
      </div>
    </>
  );
};