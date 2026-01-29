import React from "react";
import Switch from "@mui/material/Switch";
import Tooltip from "@mui/material/Tooltip";
import FormControlLabel from "@mui/material/FormControlLabel";
import { useWasm } from "../WasmContext";

const OpenCCSwitch: React.FC<{ disabled: boolean }> = ({ disabled }) => {
  const { useOpenCC, setUseOpenCC } = useWasm();

  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setUseOpenCC(event.target.checked);
  };

  return (
    <Tooltip
      title={
        <>
          Enable OpenCC dictionaries (i.e. conversion tables)
          <br />/ 啟用 OpenCC 字典（即轉換表）
        </>
      }
    >
      <FormControlLabel
        control={<Switch checked={useOpenCC} onChange={handleChange} color="secondary" disabled={disabled} />}
        label="OpenCC"
      />
    </Tooltip>
  );
};

export default OpenCCSwitch;
