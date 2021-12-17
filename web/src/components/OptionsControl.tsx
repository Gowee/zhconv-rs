import { forwardRef, ForwardedRef } from "react";

import FormGroup from "@material-ui/core/FormGroup";
import FormControlLabel from "@material-ui/core/FormControlLabel";
import Switch from "@material-ui/core/Switch";
import Grid from "@material-ui/core/Grid";
import ButtonGroup from "@material-ui/core/ButtonGroup";
import Button from "@material-ui/core/Button";
import Tooltip from "@material-ui/core/Tooltip";
import InputLabel from "@material-ui/core/InputLabel";
import MenuItem from "@material-ui/core/MenuItem";
import FormHelperText from "@material-ui/core/FormHelperText";
import FormControl from "@material-ui/core/FormControl";
import Select from "@material-ui/core/Select";
// import ChangeCircleOutlinedIcon from '@material-ui/icons/ChangeCircleOutlined';

function OptionsControl(
  {
    // ipKind,
    // toggleIpv4,
    // toggleIpv6,
    // bogonFilter,
    // toggleReservedFilter,
    handleConvert,
  }: {
    // ipKind: string;
    // toggleIpv4: () => void;
    // toggleIpv6: () => void;
    // bogonFilter?: string;
    // toggleReservedFilter: () => void;
    handleConvert: (target: string, mediawiki?: boolean) => void;
  },
  ref: ForwardedRef<any>
) {
  return (
    <Grid container ref={ref} direction="row" justifyContent="space-around" alignItems="center">
      <Grid item>
        <FormControl variant='outlined' margin='dense'>
          <InputLabel id="target-label">Target / 目標</InputLabel>
          <Select
            labelId="target-label"
            id="target-select"
            value={"zh-TW"}
            // onChange={handleChange}
          >
            <MenuItem value="zh">zh / 原文</MenuItem>
            <MenuItem value="zh-hant">zh-Hant 繁體</MenuItem>
            <MenuItem value="zh-hans">zh-Hans 简体</MenuItem>
            <MenuItem value="zh-TW">zh-TW 臺灣正體</MenuItem>
            <MenuItem value="zh-HK">zh-HK 香港繁體</MenuItem>
            <MenuItem value="zh-MO">zh-MO 澳門繁體</MenuItem>
            <MenuItem value="zh-CN">zh-CN 大陆简体</MenuItem>
            <MenuItem value="zh-SG">zh-SG 大陆简体</MenuItem>
            <MenuItem value="zh-MY">zh-MY 大陆简体</MenuItem>
          </Select>
        </FormControl>
      </Grid>
      <Grid item>
        <FormControlLabel
          control={
            <Switch
              // checked={ipKind !== "ipv6"}
              // onChange={toggleIpv4}
              name="mediawiki"
            />
          }
          label="Mediawiki Rules"
        />
      </Grid>
      <Grid item>
        <ButtonGroup color="primary" aria-label="control button group">
          <Button color="primary" onClick={() => handleConvert("zh-tw")}>
            Convert 轉換
          </Button>
          <Button color="primary" onClick={() => handleConvert("zh-cn")}>
            Reverse
          </Button>
        </ButtonGroup>
      </Grid>
      <Grid item>
        {/* <FormGroup row>
          <FormControlLabel
            control={
              <Switch
                checked={ipKind !== "ipv6"}
                onChange={toggleIpv4}
                name="ipv4"
              />
            }
            label="IPv4"
          />
          <FormControlLabel
            control={
              <Switch
                checked={ipKind !== "ipv4"}
                onChange={toggleIpv6}
                name="ipv6"
              />
            }
            label="IPv6"
          />
        </FormGroup> */}
      </Grid>
      <Grid item>
        {/* <Tooltip title="If activated, all reserved IPs for special purposes (RFC 5735 and RFC 6890) are filtered out from the output. This might be useful when reversing.">
          <FormControlLabel
            control={
              <Switch
                checked={bogonFilter === "reserved"}
                onChange={toggleReservedFilter}
                name="excludeReserved"
              />
            }
            label="Exclude reserved IPs"
          />
        </Tooltip> */}
      </Grid>
    </Grid>
  );
}

export default forwardRef(OptionsControl);
