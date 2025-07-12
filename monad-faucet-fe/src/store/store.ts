import { create } from "zustand";
import { Token } from "./TokenListStore";

interface codeState {
  code: string | null;
  setCode: (newCode: string | null) => void;
}

interface AppState {
  dripToken: Token | null;
  isLoading: boolean;
  isSuccess: boolean;
  isError: boolean;
  txHash: string | null;
  amount: string | null;
  nextAccess: string | null;
  magnification: number;
  setNextAccess: (nextAccess: string | null) => void;
  setAmount: (amount: string | null) => void;
  setLoading: (loading: boolean) => void;
  setSuccess: (success: boolean) => void;
  setError: (error: boolean) => void;
  setTxHash: (hash: string | null) => void;
  setDripToken: (token: Token | null) => void;
  setMagnification: (magnification: number) => void;
}

export type User = {
    id: String;
    username: String;
    github_id: String;
    access_token: String;
    avatar_url: String;
    email: String;
}

const useCodeStore = create<codeState>((set) => ({
  code: null,
  setCode: (newCode) => set({ code: newCode }),
}));

const useAppStore = create<AppState>((set) => ({
  dripToken: null,
  isLoading: false,
  isSuccess: false,
  isError: false,
  txHash: null,
  amount: "",
  nextAccess: null,
  magnification: 1,
  setLoading: (loading) => set({ isLoading: loading }),
  setSuccess: (success) => set({ isSuccess: success }),
  setError: (error) => set({ isError: error }),
  setTxHash: (hash) => set({ txHash: hash }),
  setAmount: (amount) => set({ amount: amount }),
  setNextAccess: (nextAccess) => set({ nextAccess: nextAccess }),
  setDripToken: (token) => set({ dripToken: token }),
  setMagnification: (magnification) => set({magnification})
}));

const useUserStore = create<{
  user: User | null;
  setUser: (newUser: User | null) => void;
}>((set) => ({
  user: null,
  setUser: (newUser) => set({ user: newUser }),
}));

const updateUser = (userDetails: User | null) => {
  const setUser = useUserStore.getState().setUser;
  setUser(userDetails);
};

export {
  useCodeStore,
  useAppStore,
  useUserStore,
  type codeState,
  type AppState,
  updateUser,
};
