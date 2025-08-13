import { useState, useEffect } from "react";

interface BowlingPinSelectorProps {
  selectedPins: number[];
  onPinsChange: (pins: number[]) => void;
}

export const BowlingPinSelector = ({
  selectedPins,
  onPinsChange,
}: BowlingPinSelectorProps) => {
  const [pins, setPins] = useState<Set<number>>(new Set(selectedPins));

  useEffect(() => {
    setPins(new Set(selectedPins));
  }, [selectedPins]);

  const togglePin = (pinNumber: number) => {
    const newPins = new Set(pins);
    if (newPins.has(pinNumber)) {
      newPins.delete(pinNumber);
    } else {
      newPins.add(pinNumber);
    }
    setPins(newPins);
    onPinsChange(Array.from(newPins).sort((a, b) => a - b));
  };

  const renderPin = (pinNumber: number) => {
    const isSelected = pins.has(pinNumber);
    const pinId = `pin-${pinNumber}`;

    return (
      <div key={pinNumber} className="relative">
        <input
          type="checkbox"
          id={pinId}
          className="pin-checkbox"
          checked={isSelected}
          onChange={() => togglePin(pinNumber)}
        />
        <label
          htmlFor={pinId}
          className="pin-label relative w-12 h-16 block cursor-pointer transition-all transform hover:scale-110"
        >
          <div className="relative w-full h-full">
            {/* Pin SVG */}
            <img
              src={
                isSelected
                  ? "/bowling_pin_enabled.svg"
                  : "/bowling_pin_disabled.svg"
              }
              alt={`Pin ${pinNumber}`}
              className="pin-svg w-full h-full relative z-10"
            />

            {/* Pin Number */}

            <div
              className={`absolute top-8 left-0 ${
                pinNumber === 10 ? "right-0" : "right-0.5"
              } flex items-center justify-center z-20 ${
                isSelected ? "text-blue-900 font-bold" : "text-gray-700"
              }`}
            >
              <span
                className={pinNumber === 10 ? "text-xs" : "text-sm"}
                style={{
                  textShadow: isSelected
                    ? "0 0 4px rgba(255,255,255,0.9)"
                    : "0 0 3px rgba(255,255,255,0.7)",
                }}
              >
                {pinNumber}
              </span>
            </div>
          </div>
        </label>
      </div>
    );
  };

  return (
    <div className="space-y-2">
      <label className="block text-sm font-medium text-neutral mb-4">
        Select Pins
      </label>

      {/* Bowling Pin Formation */}
      <div className="flex flex-col items-center space-y-2">
        {/* Row 1: Pins 7-10 */}
        <div className="flex space-x-2">{[7, 8, 9, 10].map(renderPin)}</div>

        {/* Row 2: Pins 4-6 */}
        <div className="flex space-x-2">{[4, 5, 6].map(renderPin)}</div>

        {/* Row 3: Pins 2-3 */}
        <div className="flex space-x-2">{[2, 3].map(renderPin)}</div>

        {/* Row 4: Pin 1 */}
        <div className="flex space-x-2">{[1].map(renderPin)}</div>
      </div>
    </div>
  );
};
