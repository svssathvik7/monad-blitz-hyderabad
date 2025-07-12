import { useEffect } from "react";
import { useTokenListStore } from "../store/TokenListStore";
import { API } from "../constants/api";

const useTokenFetcher = () => {
  const { setTokens } = useTokenListStore();

  const fetchTokens = async () => {
    try {
      const response = await fetch(API().tokens);
      const data = await response.json();

      if (data.status === "Success") setTokens(data.data);
    } catch (error) {
      console.error("Error fetching tokens:", error);
    }
  };

  useEffect(() => {
    fetchTokens();

    const intervalId = setInterval(() => fetchTokens(), 20000);

    return () => clearInterval(intervalId);
  }, [setTokens]);
};

export default useTokenFetcher;
