import { modalNames, modalStore } from "../../store/ModalStore";
import { NewToken } from "./newToken";
import { TokenRefreshment } from "./TokenRefreshment";

export const ModalComp = () => {
  const { modalName, setCloseModal } = modalStore();

  return (
    <div className="">
      <NewToken
        open={modalName.createToken}
        onClose={() => setCloseModal(modalNames.createToken)}
      />
      <TokenRefreshment
        open={modalName.tokenRefreshments}
        onClose={() => setCloseModal(modalNames.tokenRefreshments)}
      />
    </div>
  );
};
