import { invoke } from "@tauri-apps/api/core";
import { Balance, Transaction } from "../types";

export const useTauri = () => {
  const getBalances = async (): Promise<Balance[]> => {
    return await invoke("balances");
  };

  const getNames = async (): Promise<string[]> => {
    return await invoke("names");
  };

  const getTransactions = async (count: number): Promise<Transaction[]> => {
    return await invoke("transactions", { count });
  };

  const createSplit = async (
    name: string,
    splitString: string
  ): Promise<void> => {
    return await invoke("create_split", { name, split_string: splitString });
  };

  const convertSplit = async (
    name: string,
    splitString: string
  ): Promise<void> => {
    return await invoke("convert_split", { name, split_string: splitString });
  };

  const getValidSplits = async (): Promise<string[]> => {
    const splits = await invoke("get_valid_splits");
    return Array.from(splits as Set<string>);
  };

  const removeLastTransaction = async (): Promise<void> => {
    return await invoke("remove_last_transaction");
  };

  const setSheetId = async (sheetId: string): Promise<void> => {
    return await invoke("set_sheet_id", { sheetId });
  };

  const getSheetId = async (): Promise<string> => {
    return await invoke("get_sheet_id");
  };

  const setDemoSheetId = async (): Promise<void> => {
    return await invoke("set_demo_sheet_id");
  };

  return {
    getBalances,
    getNames,
    getTransactions,
    createSplit,
    convertSplit,
    getValidSplits,
    removeLastTransaction,
    setSheetId,
    getSheetId,
    setDemoSheetId,
  };
};
