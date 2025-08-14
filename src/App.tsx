import { useState, useEffect } from "react";
import { useTauri } from "./hooks/useTauri";
import { Balance, Transaction } from "./types";
import { TransactionList } from "./components/TransactionList";
import { SplitForm } from "./components/SplitForm";
import { Settings } from "./components/Settings";
import { BalanceTicker } from "./components/BalanceTicker";
import { Modal } from "./components/Modal";
import { LoadingSpinner } from "./components/LoadingSpinner";

function App() {
  const [balances, setBalances] = useState<Balance[]>([]);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [names, setNames] = useState<string[]>([]);
  const [validSplits, setValidSplits] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);

  const tauri = useTauri();

  const loadData = async (isRefresh = false) => {
    try {
      if (isRefresh) {
        setRefreshing(true);
      } else {
        setLoading(true);
      }

      const [balancesData, transactionsData, namesData, splitsData] =
        await Promise.all([
          tauri.getBalances(),
          tauri.getTransactions(10),
          tauri.getNames(),
          tauri.getValidSplits(),
        ]);

      setBalances(balancesData);
      setTransactions(transactionsData);
      setNames(namesData);
      setValidSplits(splitsData);
      setError(null);
    } catch (err) {
      setError("Failed to load data. Please check your connection.");
      console.error("Error loading data:", err);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleCreateSplit = async (name: string, split: string) => {
    await tauri.createSplit(name, split);
    await loadData(true);
    setIsModalOpen(false);
  };

  const openCreateModal = () => {
    setIsModalOpen(true);
  };

  const handleConvertFromTransaction = async (transaction: Transaction) => {
    await tauri.convertSplit(transaction.creditor, transaction.split);
    await loadData(true);
  };

  const handleRemoveLastTransaction = async () => {
    await tauri.removeLastTransaction();
    await loadData(true);
  };

  if (loading) {
    return (
      <div className="min-h-screen wood-background flex items-center justify-center">
        <div className="text-center bg-secondary bg-opacity-80 p-8 rounded-lg">
          <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-primary mx-auto mb-4"></div>
          <p
            className="text-primary"
            style={{ fontFamily: "Minecart LCD, monospace" }}
          >
            Loading Split Happens...
          </p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen wood-background flex items-center justify-center">
        <div className="text-center bg-secondary p-8 rounded-lg shadow-md">
          <div
            className="text-negative text-xl mb-4"
            style={{ fontFamily: "Minecart LCD, monospace" }}
          >
            ⚠️ Error
          </div>
          <p
            className="text-negative mb-4"
            style={{ fontFamily: "Minecart LCD, monospace" }}
          >
            {error}
          </p>
          <button
            onClick={() => loadData()}
            className="bg-primary-hover text-primary px-4 py-2 rounded-md"
            style={{ fontFamily: "Minecart LCD, monospace" }}
          >
            Try Again
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen wood-background">
      <div className="container mx-auto px-2 py-4 pt-10 pb-10">
        <header className="text-center mb-8 relative">
          <div className="bg-neutral rounded-lg p-4 inline-block">
            <h1 className="text-4xl font-bold text-neutral mb-2">
              Split Happens
            </h1>
          </div>

          <div className="absolute top-0 right-0">
            <Settings
              onSheetIdChange={tauri.setSheetId}
              onSetDemoSheetId={tauri.setDemoSheetId}
              getCurrentSheetId={tauri.getSheetId}
            />
          </div>
        </header>

        {/* Main Content */}
        <div className="max-w-4xl mx-auto">
          {/* Add Split Button */}
          <div className="text-center mb-8">
            <button
              onClick={openCreateModal}
              className="bg-primary-hover text-primary px-8 py-3 rounded-lg font-semibold text-lg transition-colors shadow-lg"
            >
              Add Split
            </button>
          </div>

          {/* Transactions List */}
          {refreshing ? (
            <LoadingSpinner />
          ) : (
            <TransactionList
              transactions={transactions}
              onUndo={handleRemoveLastTransaction}
              onConvert={handleConvertFromTransaction}
              onRefresh={() => loadData(true)}
              refreshing={refreshing}
            />
          )}
        </div>

        <BalanceTicker balances={balances} />

        {/* Modal */}
        <Modal
          isOpen={isModalOpen}
          onClose={() => setIsModalOpen(false)}
          title="Add Split"
        >
          <SplitForm
            names={names}
            validSplits={validSplits}
            onCreateSplit={handleCreateSplit}
          />
        </Modal>
      </div>
    </div>
  );
}

export default App;
