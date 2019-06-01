import vapoursynth as vs
core = vs.core

#black = core.std.BlankClip(format=vs.YUV444P8, width=1920, height=1080, color=[0, 128, 128])
#white = core.std.BlankClip(black, color=[255, 128, 128])
core.std.LoadPlugin('target/release/libadaptivegrain_rs.so')
zweihu = core.ffms2.Source('1558625524644.jpg').resize.Point(format=vs.YUV444PS).std.PlaneStats() * 200
#grained = white.std.PlaneStats().adg.Mask()
grained = zweihu.adg.Mask()
#grained = zweihu
#grained = white.grain.Add(2, constant=True)
#for i in range(len(grained)):
#    grained.get_frame(i)
#grained.resize.Bicubic(format=vs.RGB24, matrix_in_s='709').imwri.Write('png', 'grain%d.png').get_frame(0)
grained.set_output()
#zweihu.set_output()
