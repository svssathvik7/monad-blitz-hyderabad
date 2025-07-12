import { useQuery } from "@tanstack/react-query";
import { useEffect } from "react";
import { useCodeStore } from "../store/store.ts";
import { fetchUser, getCode } from "../utils/utils.ts";

function useFetchUser() {
  const { code, setCode } = useCodeStore();

  useEffect(() => {
    if (!code) {
      const newCode = getCode();
      if (newCode) setCode(newCode);
    }
  }, [code]);

  return useQuery({
    queryKey: ["user", { code }],
    refetchOnWindowFocus: false,
    queryFn: () => fetchUser({ code, setCode }),
    retry: 1,
  });
}

export { useFetchUser };
