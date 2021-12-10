import {useMemo} from 'react';
import Grid from "@material-ui/core/Grid";
import Typography from "@material-ui/core/Typography";
import Box from "@material-ui/core/Box";

import { countLines } from "../utils";

function Partial({ name, status }: { name: string; status: any }) {
  return (
    <Typography variant="caption" color="textSecondary">
      {name}: {status?.line_count_before ?? 0} <abbr title="Lines">L</abbr> /{" "}
      {status?.address_count_before ?? "0"}
      &nbsp;&nbsp;➟&nbsp;&nbsp;
      <b>{status?.line_count_after ?? 0}</b>
      <abbr title="Lines">L</abbr> / <b>{status?.address_count_after ?? "0"}</b>
    </Typography>
  );
}

export default function OutputStatusLine({ output }: { output: any }) {
  return (
    <Typography variant="caption" color="textSecondary">
      Lines: {useMemo(() => countLines(output), [output])} / Chars: { output ? output.length : 0}
    </Typography>
    // <Grid container direction="row" justifyContent="space-between">
    //   <Grid item>
    //     <Partial name="IPv4" status={output?.v4} />
    //   </Grid>
    //   <Grid item>
    //     <Partial name="IPv6" status={output?.v6} />
    //   </Grid>
    // </Grid>
  );
}
