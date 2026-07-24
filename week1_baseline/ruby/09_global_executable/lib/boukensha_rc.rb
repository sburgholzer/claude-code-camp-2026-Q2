# BoukenshaRc parses ~/.boukensharc, a small persistent config file that can
# set BOUKENSHA_PATH and/or BOUKENSHA_DIR so you don't have to export them
# in every shell session.
#
# Format — one KEY=VALUE per line, "#" comments and blank lines ignored:
#
#   BOUKENSHA_PATH=~/Sites/boukensha/07_the_repl_loop
#   BOUKENSHA_DIR=~/projects/mybot/.boukensha
#
# Legacy format: a file containing just a bare path (no "=" on any
# non-comment line) is treated as BOUKENSHA_PATH, e.g.:
#
#   echo ~/Sites/boukensha/07_the_repl_loop > ~/.boukensharc
module BoukenshaRc
  PATH = File.expand_path("~/.boukensharc")

  # Returns a Hash of the keys set in ~/.boukensharc, e.g.
  # {"BOUKENSHA_PATH" => "...", "BOUKENSHA_DIR" => "..."}.
  # Returns {} if the file doesn't exist or is empty.
  def self.read
    return {} unless File.exist?(PATH)

    lines = File.readlines(PATH).map(&:strip).reject { |l| l.empty? || l.start_with?("#") }
    return {} if lines.empty?

    if lines.none? { |l| l.include?("=") }
      # Legacy format: the whole file is a bare BOUKENSHA_PATH value.
      return { "BOUKENSHA_PATH" => lines.join(" ").strip }
    end

    lines.each_with_object({}) do |line, config|
      key, value = line.split("=", 2)
      next unless key && value

      config[key.strip] = value.strip
    end
  end
end
