import { NewTokenComponent } from "../faucet/NewTokenComponent";
import { Modal } from "../ui/Modal";
import { FC } from "react";

type NewTokenProps = {
  open: boolean;
  onClose: () => void;
};

export const NewToken: FC<NewTokenProps> = ({ open, onClose }) => {
  return (
    <Modal open={open} onClose={onClose}>
      <NewTokenComponent onClose={onClose} />
    </Modal>
  );
};
