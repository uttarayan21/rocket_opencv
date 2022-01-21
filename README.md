# Rust Rocket Endpoint for gaussian blur opencv 



You can send the image and the arguments as `multipart/form-data`  

In this case the file is written to disk* before reading so it might be slower than the other one.  
However In case of linux systems the `std::env::temp_dir()` is /tmp which is usually a ramdisk and  
I'm writing the file there so there **should** be no slowdowns but I don't know for sure in Windows/MacOS  

```bash
curl -X POST http://127.0.0.1:8000/blur -F 'ksize_height=45' -F 'ksize_width=45' -F 'sigma_x=0'  -F 'image=@image.png' -o image_blurred.png
```

You can also specify the non-default arguments  
format is considered ".png" by default and sigma_y is considered 0  
```bash
curl -X POST http://127.0.0.1:8000/blur -F 'ksize_height=45' -F 'ksize_width=45' -F 'sigma_x=0' -F 'sigma_y=0' -F 'format=.png' -F 'image=@image.png' -o image_blurred.png
```


You can also send the image as a base64 encoded data like so  
```bash
base64 image.png | curl -X POST http://127.0.0.1:8000/blur -d @- | base64 -d > image_blurred.png
```
