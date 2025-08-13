export const LoadingSpinner = () => {
  return (
    <div className="bg-neutral bg-opacity-90 rounded-lg shadow-md p-8">
      <div className="flex items-center justify-center">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
        <span className="ml-3 text-neutral" style={{ fontFamily: 'Minecart LCD, monospace' }}>
          Loading...
        </span>
      </div>
    </div>
  );
};