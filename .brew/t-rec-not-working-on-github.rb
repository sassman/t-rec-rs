class TRec < Formula
  desc "Blazingly fast terminal recorder that generates animated gif images for the web"
  homepage "https://github.com/sassman/t-rec-rs"
  url "https://github.com/sassman/t-rec-rs/archive/v0.2.1.tar.gz"
  sha256 "a23ae0e19b76740220f4640a82d2813fe1002609bbf28ab180f3fc9228b141e3"
  license "GPL-3.0-only"

  depends_on "rust" => :build
  depends_on "imagemagick"

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    # let's fetch the window id
    o = `#{bin}/t-rec -l | tail -1`
    win_id = o.split(/\s|\n/)[-1]
    # verify that it's an appropriate id
    raise "No window id retrieved" unless win_id && Integer(win_id).positive?

    # seems that grabbing a window does not work on github workers. Locally that just works fine
    lets record the window
    p "Going to record window id #{win_id}"
    input, _, thread = Open3.popen2("WINDOWID=#{win_id} #{bin}/t-rec")
    sleep 1
    input.puts "# echo foo"
    sleep 1
    input.close
    we wait until the gif has been generated
    sleep 5
    now there should be a gif
    assert_predicate testpath/"t-rec.gif", :exist?
  ensure
    if thread
     begin
        just in case something went wrong
       Process.kill("TERM", thread.pid)
     rescue Errno::ESRCH
        # all good, no issue
     end
    end
  end
end
