#-------------------------------------------------------------------------------
# author:	Nikita Makarevich
# email:	nikita.makarevich@spbpu.com
# 2021
#-------------------------------------------------------------------------------
# Mouse Brain View
#-------------------------------------------------------------------------------

from PIL import Image
filename = r'window_image.png'
img = Image.open(filename)
img.save('window_image.ico')
