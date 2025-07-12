import { Button } from "../ui/Button";
import { modalNames, modalStore } from "../../store/ModalStore";

export const CisternToken = () => {
  const { setOpenModal } = modalStore();

  const handleOpenCreateTokenSelector = () =>
    setOpenModal(modalNames.createToken);

  return (
    <div className="to-blue-50 z-20 mx-auto flex max-w-md flex-col items-center justify-center rounded-2xl border border-gray-100 bg-gradient-to-br from-white via-gray-50 px-8 py-10 shadow-2xl">
      <div className="mb-6 flex flex-col items-center">
        <p className="mt-2 text-center text-base text-gray-500">
          You can create your own token and use this faucet to fill it with
          testnet tokens.
        </p>
      </div>
      <Button
        handleClick={handleOpenCreateTokenSelector}
        className={`w-full rounded-xl bg-purple-950 py-2 font-semibold text-white shadow transition hover:bg-purple-800`}
      >
        Create Token
      </Button>
    </div>
  );
};
