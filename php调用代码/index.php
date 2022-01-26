<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Document</title>
</head>
<body>
    <h3>选择你的fbo格式文件</h3>
    <form action="" method="post" enctype="multipart/form-data">
        <input type="file" name="fbo">
        <input type="submit">
    </form>


<?php
if($_FILES){
$execute_file = "./fbo";

$fbo_file = $_FILES['fbo']['tmp_name'];
exec("\"$execute_file\" \"$fbo_file\"", $retval);

?>
<h3>解析结果</h3>
<textarea name="" style="width:95vw; height:75vh;border:1px solid #999;" readonly="readonly" id="" cols="30" rows="10"><?=$retval[0]?></textarea>
<?php } ?>
</body>
</html>
