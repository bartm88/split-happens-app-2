import { useState, useEffect } from "react";
import { BowlingPinSelector } from "./BowlingPinSelector";

interface SplitFormProps {
  names: string[];
  validSplits: string[];
  onCreateSplit: (name: string, split: string) => Promise<void>;
  initialName?: string;
  initialSplit?: string;
}

export const SplitForm = ({
  names,
  validSplits,
  onCreateSplit,
  initialName = "",
  initialSplit = "",
}: SplitFormProps) => {
  const [selectedName, setSelectedName] = useState(initialName);
  const [selectedSplit, setSelectedSplit] = useState(initialSplit);
  const [selectedPins, setSelectedPins] = useState<number[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Filter out "Pot" from names
  const playerNames = names.filter((name) => name !== "Pot");

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

  const createSplitString = (pins: number[]): string => {
    if (pins.length === 0) return "";
    return pins.join("-");
  };

  const isValidSplit = (pins: number[]): boolean => {
    if (pins.length === 0) return false;
    const splitString = createSplitString(pins);
    return validSplits.includes(splitString);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!selectedName || selectedPins.length === 0) return;

    // Validate the split
    if (!isValidSplit(selectedPins)) {
      alert("Invalid split combination. Please select a valid split.");
      return;
    }

    const splitString = createSplitString(selectedPins);

    setIsSubmitting(true);
    try {
      await onCreateSplit(selectedName, splitString);
    } catch (error) {
      console.error("Error submitting split:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label
            htmlFor="name"
            className="block text-sm font-medium text-neutral mb-2"
          >
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

        <BowlingPinSelector
          selectedPins={selectedPins}
          onPinsChange={setSelectedPins}
        />

        <button
          type="submit"
          disabled={
            isSubmitting ||
            !selectedName ||
            selectedPins.length === 0 ||
            !isValidSplit(selectedPins)
          }
          className={`w-full py-2 px-4 rounded-md font-medium transition-colors bg-primary-hover text-primary disabled:opacity-50 disabled:cursor-not-allowed`}
          title={
            selectedPins.length > 0 && !isValidSplit(selectedPins)
              ? "Invalid split"
              : ""
          }
        >
          {isSubmitting ? "Processing..." : "Add Split"}
        </button>
      </form>
    </div>
  );
};
