import React from "react";
import Switch from "@mui/material/Switch";
import Tooltip from "@mui/material/Tooltip";
import FormControlLabel from "@mui/material/FormControlLabel";
import { useApp } from "../AppContext";

const OpenCCSwitch: React.FC<{ disabled: boolean }> = ({ disabled }) => {
  const { useOpenCC, setUseOpenCC } = useApp();

  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setUseOpenCC(event.target.checked);
  };

  return (
    <Tooltip
      title={
        <>
          Enable additional OpenCC dictionaries
          <br />/ 啟用額外的 OpenCC 字典
        </>
      }
    >
      <FormControlLabel
        control={
          <Switch
            checked={useOpenCC}
            onChange={handleChange}
            color="secondary"
            disabled={disabled}
          />
        }
        label="OpenCC"
      />
    </Tooltip>
  );
};

export default OpenCCSwitch;
