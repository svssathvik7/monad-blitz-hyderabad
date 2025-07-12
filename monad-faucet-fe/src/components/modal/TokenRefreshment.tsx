import { FC } from "react";
import { Modal } from "../ui/Modal";
import { TokenRefreshmentComponent } from "../faucet/TokenRefreshmentComponent";

type TokenRefreshmentProps = {
  open: boolean;
  onClose: () => void;
};

export const TokenRefreshment: FC<TokenRefreshmentProps> = ({
  onClose,
  open,
}) => {
  return (
    <Modal open={open} onClose={onClose}>
        <TokenRefreshmentComponent onClose={onClose} />
    </Modal>
  );
};
