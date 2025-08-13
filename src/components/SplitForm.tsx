import { useState, useEffect } from 'react';
import { BowlingPinSelector } from './BowlingPinSelector';

interface SplitFormProps {
  names: string[];
  validSplits: string[];
  onCreateSplit: (name: string, split: string) => Promise<void>;
  onConvertSplit: (name: string, split: string) => Promise<void>;
  initialMode?: 'create' | 'convert';
  initialName?: string;
  initialSplit?: string;
}

export const SplitForm = ({ 
  names, 
  validSplits, 
  onCreateSplit, 
  onConvertSplit,
  initialMode = 'create',
  initialName = '',
  initialSplit = ''
}: SplitFormProps) => {
  const [selectedName, setSelectedName] = useState(initialName);
  const [selectedSplit, setSelectedSplit] = useState(initialSplit);
  const [selectedPins, setSelectedPins] = useState<number[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [mode, setMode] = useState<'create' | 'convert'>(initialMode);
  
  // Filter out "Pot" from names
  const playerNames = names.filter(name => name !== 'Pot');

  useEffect(() => {
    if (initialName) {
      setSelectedName(initialName);
    } else if (playerNames.length > 0 && !selectedName) {
      setSelectedName(playerNames[0]);
    }
  }, [playerNames, selectedName, initialName]);

  useEffect(() => {
    if (initialSplit) {
      setSelectedSplit(initialSplit);
    } else if (validSplits.length > 0 && !selectedSplit) {
      setSelectedSplit(validSplits[0]);
    }
  }, [validSplits, selectedSplit, initialSplit]);

  useEffect(() => {
    setMode(initialMode);
  }, [initialMode]);

  const createSplitString = (pins: number[]): string => {
    if (pins.length === 0) return '';
    return pins.join('-');
  };

  const isValidSplit = (pins: number[]): boolean => {
    if (pins.length === 0) return false;
    const splitString = createSplitString(pins);
    return validSplits.includes(splitString);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (mode === 'create') {
      if (!selectedName || selectedPins.length === 0) return;
      
      // Validate the split
      if (!isValidSplit(selectedPins)) {
        alert('Invalid split combination. Please select a valid split.');
        return;
      }
      
      const splitString = createSplitString(selectedPins);
      
      setIsSubmitting(true);
      try {
        await onCreateSplit(selectedName, splitString);
      } catch (error) {
        console.error('Error submitting split:', error);
      } finally {
        setIsSubmitting(false);
      }
    } else {
      if (!selectedName || !selectedSplit) return;
      
      setIsSubmitting(true);
      try {
        await onConvertSplit(selectedName, selectedSplit);
      } catch (error) {
        console.error('Error submitting split:', error);
      } finally {
        setIsSubmitting(false);
      }
    }
  };

  return (
    <div>
      {initialMode === undefined && (
        <h2 className="text-xl font-semibold text-neutral mb-4">
          {mode === 'create' ? 'Add Split' : 'Convert Split'}
        </h2>
      )}
      
      {/* Only show mode switcher when not in modal (initialMode undefined) */}
      {initialMode === undefined && (
        <div className="mb-4">
          <div className="flex space-x-2">
            <button
              type="button"
              onClick={() => setMode('create')}
              className={`px-4 py-2 rounded-md font-medium transition-colors ${
                mode === 'create'
                  ? 'bg-primary text-primary'
                  : 'bg-neutral text-neutral hover:bg-neutral-hover'
              }`}
            >
              Add Split
            </button>
            <button
              type="button"
              onClick={() => setMode('convert')}
              className={`px-4 py-2 rounded-md font-medium transition-colors ${
                mode === 'convert'
                  ? 'bg-confirmation text-confirmation'
                  : 'bg-neutral text-neutral hover:bg-neutral-hover'
              }`}
            >
              Convert Split
            </button>
          </div>
        </div>
      )}

      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label htmlFor="name" className="block text-sm font-medium text-neutral mb-2">
            Player
          </label>
          <select
            id="name"
            value={selectedName}
            onChange={(e) => setSelectedName(e.target.value)}
            className="w-full px-3 py-2 border border-neutral rounded-md bg-neutral text-neutral focus:outline-none focus-ring focus:ring-2"
          >
            {playerNames.map((name) => (
              <option key={name} value={name}>
                {name}
              </option>
            ))}
          </select>
        </div>

        {mode === 'create' ? (
          <BowlingPinSelector
            selectedPins={selectedPins}
            onPinsChange={setSelectedPins}
          />
        ) : (
          <div>
            <label htmlFor="split" className="block text-sm font-medium text-neutral mb-2">
              Split Type
            </label>
            <select
              id="split"
              value={selectedSplit}
              onChange={(e) => setSelectedSplit(e.target.value)}
              className="w-full px-3 py-2 border border-neutral rounded-md bg-neutral text-neutral focus:outline-none focus-ring focus:ring-2"
            >
              {validSplits.map((split) => (
                <option key={split} value={split}>
                  {split}
                </option>
              ))}
            </select>
          </div>
        )}

        <button
          type="submit"
          disabled={isSubmitting || !selectedName || (mode === 'create' ? selectedPins.length === 0 || !isValidSplit(selectedPins) : !selectedSplit)}
          className={`w-full py-2 px-4 rounded-md font-medium transition-colors ${
            mode === 'create'
              ? 'bg-primary-hover text-primary'
              : 'bg-confirmation-hover text-confirmation'
          } disabled:opacity-50 disabled:cursor-not-allowed`}
          title={mode === 'create' && selectedPins.length > 0 && !isValidSplit(selectedPins) ? 'Invalid split' : ''}
        >
          {isSubmitting ? 'Processing...' : mode === 'create' ? 'Add Split' : 'Convert Split'}
        </button>
      </form>
    </div>
  );
};