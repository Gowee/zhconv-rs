import { forwardRef, ForwardedRef } from "react";

import FormGroup from "@material-ui/core/FormGroup";
import FormControlLabel from "@material-ui/core/FormControlLabel";
import Switch from "@material-ui/core/Switch";
import Grid from "@material-ui/core/Grid";
import ButtonGroup from "@material-ui/core/ButtonGroup";
import Button from "@material-ui/core/Button";
import Tooltip from "@material-ui/core/Tooltip";

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
    <Grid container ref={ref} direction="row" justifyContent="space-around">
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
