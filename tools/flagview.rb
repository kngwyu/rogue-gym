# In the source of rogue, bit flags is defined like
#define F_PASS		0x80		/* is a passageway */
#define F_SEEN		0x40		/* have seen this spot before */
#define F_DROPPED	0x20		/* object was dropped here */
#define F_LOCKED	0x20		/* door is locked */
#define F_REAL		0x10		/* what you see is what you get */
#define F_PNUM		0x0f		/* passage number mask */
#define F_TMASK		0x07		/* trap number mask */
# so let's convert them into binary for checking

inputs = STDIN.readlines.select{|str| str.start_with?("#define")}.map do |input|
  input = input.strip
  defines = input.split(" ")
  if defines.length > 2 then
    num = if defines[2].start_with?("0x") then
            defines[2].to_i(16)
          else
            defines[2].to_i
          end
    [defines[1], num]
  else
    []
  end
end

max_flag_len = inputs.inject(0) {|max, elem| [max, elem[0].length].max}

inputs.each do |input|
  len = input[0].length
  out = input[0] + " " * (max_flag_len - len + 1)
  puts out + sprintf("%020b", input[1])
end
