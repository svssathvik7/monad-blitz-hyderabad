import { Faucet } from "./components/faucet/Faucet";
import { Layout } from "./layout";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import useTokenFetcher from "./hooks/useTokenFetcher";
import { ModalComp } from "./components/modal/ModalComp";

const queryClient = new QueryClient();

function App() {
  useTokenFetcher();

  return (
    <>
      <QueryClientProvider client={queryClient}>
        <Layout>
          <ModalComp />
          <Faucet />
        </Layout>
      </QueryClientProvider>
    </>
  );
}

export default App;
