import { useTheme } from '../contexts/ThemeContext';

export const ThemeSwitcher = () => {
  const { theme, setTheme } = useTheme();

  const themes = [
    { value: 'amber', label: 'Amber', color: 'bg-amber-500' },
    { value: 'red', label: 'Red', color: 'bg-red-500' },
    { value: 'green', label: 'Green', color: 'bg-green-500' },
    { value: 'neon', label: 'Neon', color: 'bg-purple-500' },
  ] as const;

  return (
    <div className="flex items-center space-x-2">
      <span className="text-neutral text-sm font-medium">Theme:</span>
      <div className="flex space-x-1">
        {themes.map((t) => (
          <button
            key={t.value}
            onClick={() => setTheme(t.value as any)}
            className={`w-8 h-8 rounded-full ${t.color} transition-transform hover:scale-110 ${
              theme === t.value ? 'ring-2 ring-offset-2 ring-offset-black ring-white' : ''
            }`}
            title={t.label}
          />
        ))}
      </div>
    </div>
  );
};