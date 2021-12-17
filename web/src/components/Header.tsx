import Typography from "@material-ui/core/Typography";
import Box from "@material-ui/core/Box";
import Popover from "@material-ui/core/Popover"; // TODO:
import InfoOutlinedIcon from "@material-ui/icons/InfoOutlined";

export default function Header() {
  return (
    <header>
      <Typography variant="h3" component="h1" gutterBottom>
        zhconv-rs 中文简繁及地區詞轉換
      </Typography>
      <Box display="flex" alignItems="center">
        <Typography variant="h6" component="h2" gutterBottom>
          {
            "Convert Chinese among different variants / 轉換简、繁體以及兩岸四地、新马的地區詞"
          }
        </Typography>
        <InfoOutlinedIcon color="primary" />
      </Box>
      {/* <Typography  component="h3" gutterBottom>
        {"Based on conversion tables maintained by Chinese Wikipedia / 基於中文維基百科維護的轉換規則"}
      </Typography> */}
    </header>
  );
}
