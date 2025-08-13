import { useState, useEffect } from 'react';
import { ThemeSwitcher } from './ThemeSwitcher';

interface SettingsProps {
  onSheetIdChange: (sheetId: string) => Promise<void>;
  onSetDemoSheetId: () => Promise<void>;
  getCurrentSheetId: () => Promise<string>;
}

export const Settings = ({ onSheetIdChange, onSetDemoSheetId, getCurrentSheetId }: SettingsProps) => {
  const [sheetId, setSheetId] = useState('');
  const [currentSheetId, setCurrentSheetId] = useState('');
  const [isOpen, setIsOpen] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    const loadCurrentSheetId = async () => {
      const current = await getCurrentSheetId();
      setCurrentSheetId(current);
      setSheetId(current);
    };
    loadCurrentSheetId();
  }, [getCurrentSheetId]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!sheetId.trim()) return;

    setIsSaving(true);
    try {
      await onSheetIdChange(sheetId.trim());
      setCurrentSheetId(sheetId.trim());
      setIsOpen(false);
    } catch (error) {
      console.error('Error updating sheet ID:', error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleSetDemo = async () => {
    setIsSaving(true);
    try {
      await onSetDemoSheetId();
      const newSheetId = await getCurrentSheetId();
      setCurrentSheetId(newSheetId);
      setSheetId(newSheetId);
      setIsOpen(false);
    } catch (error) {
      console.error('Error setting demo sheet ID:', error);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="bg-secondary-hover text-secondary p-2 rounded-md transition-colors w-10 h-10 flex items-center justify-center"
        title="Settings"
      >
        ⚙️
      </button>

      {isOpen && (
        <>
          {/* Backdrop */}
          <div 
            className="fixed inset-0 z-20"
            style={{ backgroundColor: 'rgba(0, 0, 0, 0.6)' }}
            onClick={() => setIsOpen(false)}
          />
          
          {/* Settings Panel */}
          <div className="fixed inset-0 flex items-center justify-center z-30 p-4 pointer-events-none">
            <div className="bg-neutral rounded-lg shadow-2xl border border-neutral p-4 w-96 max-h-[90vh] overflow-auto pointer-events-auto">
              <div className="flex justify-between items-center mb-4">
                <h3 className="text-lg font-semibold text-neutral">Settings</h3>
                <button
                  onClick={() => setIsOpen(false)}
                  className="text-neutral hover:text-negative transition-colors text-2xl leading-none"
                >
                  ×
                </button>
              </div>
          
          <div className="mb-6">
            <label className="block text-sm font-medium text-neutral mb-2">
              Theme
            </label>
            <ThemeSwitcher />
          </div>
          
          <div className="mb-4">
            <label className="block text-sm font-medium text-neutral mb-1">
              Current Sheet ID:
            </label>
            <p className="text-sm text-neutral opacity-80 bg-neutral p-2 rounded break-all">
              {currentSheetId || 'Not set'}
            </p>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label htmlFor="sheet-id" className="block text-sm font-medium text-neutral mb-2">
                Google Sheet ID
              </label>
              <input
                id="sheet-id"
                type="text"
                value={sheetId}
                onChange={(e) => setSheetId(e.target.value)}
                placeholder="Enter Google Sheet ID..."
                className="w-full px-3 py-2 border border-neutral rounded-md bg-neutral text-neutral focus:outline-none focus-ring focus:ring-2"
              />
            </div>

            <div className="flex space-x-2">
              <button
                type="submit"
                disabled={isSaving || !sheetId.trim() || sheetId === currentSheetId}
                className="flex-1 bg-primary-hover text-primary py-2 px-4 rounded-md font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isSaving ? 'Saving...' : 'Update'}
              </button>
              
              <button
                type="button"
                onClick={handleSetDemo}
                disabled={isSaving}
                className="flex-1 bg-confirmation-hover text-confirmation py-2 px-4 rounded-md font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Use Demo Sheet
              </button>
            </div>

          </form>
            </div>
          </div>
        </>
      )}
    </div>
  );
};